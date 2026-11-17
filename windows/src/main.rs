#![windows_subsystem = "windows"]

mod actions;
mod avatar;
mod config;
mod dialog;
mod embed;
mod gfx;
mod session;
mod tooltip;
mod tray;
mod ui;
mod util;
mod watcher;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::System::Threading::{CreateMutexW, OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};
use windows_sys::Win32::UI::Controls::Dialogs::*;
use windows_sys::Win32::UI::HiDpi::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use config::{Account, Config, COLORS};
use gfx::*;
use session::{Session, State};
use util::{initials, log, w};

use actions::*;
use embed::*;
use tooltip::*;
use ui::*;

pub const APP_NAME: &str = "Betacom";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEVELOPER: &str = "Betacom";
pub const DEVELOPER_URL: &str = "https://github.com/betacomagency2026/BetaZalo";
pub const UPDATE_URL: &str = "https://github.com/betacomagency2026/BetaZalo/releases";

const MAIN_CLASS: &str = "BetacomMain";
const PANE_CLASS: &str = "BetacomPane";
const TIMER_TICK: usize = 1;
const WM_MOUSELEAVE_X: u32 = 0x02A3;
const HSHELL_FLASH_X: usize = 0x8006;
const NOTIFY_COOLDOWN: Duration = Duration::from_secs(8);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
/// thông điệp nội bộ: trao quyền bàn phím cho cửa sổ Zalo đang chọn
const WM_APP_FOCUS: u32 = 0x8000 + 20;
const WM_APP_RELAYOUT: u32 = 0x8000 + 21;
const WM_APP_PURGE_FAILED: u32 = 0x8000 + 22;
const SAVE_DELAY: Duration = Duration::from_secs(2);

static MAIN_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

fn hwnd_main_any() -> HWND {
    MAIN_HWND.load(std::sync::atomic::Ordering::Relaxed) as HWND
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Act {
    Start,
    Stop,
    Restart,
    Reattach,
    Settings,
    Download,
    PickExe,
    AddAcc,
    Developer,
    Toggle,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Hit {
    None,
    Account(usize),
    Btn(Act),
}

struct Pending {
    count: u32,
    title: String,
    text: String,
}

struct Stopping {
    job: HANDLE,
    since: Instant,
    restart: bool,
}

struct App {
    hwnd: HWND,
    pane: HWND,
    cfg: Config,
    sessions: HashMap<String, Session>,
    stopping: HashMap<String, Stopping>,
    failed: HashMap<String, String>,
    zalo: Option<PathBuf>,
    dpi: u32,
    fonts: Fonts,
    hits: Vec<(RECT, Hit)>,
    pane_hits: Vec<(RECT, Hit)>,
    hover: Hit,
    pane_hover: Hit,
    queue: VecDeque<String>,
    last_auto: Instant,
    sig: String,
    // tin mới / thông báo
    has_new: HashSet<String>,
    last_notify: HashMap<String, Instant>,
    pending: HashMap<String, Pending>,
    flash_pending: HashMap<String, Instant>,
    last_notified_id: Option<String>,
    hidden_to_tray: bool,
    avatars: HashMap<String, Image>,
    // danh sách tài khoản
    scroll: usize,
    list_rect: RECT,
    tip: HWND,
    save_due: Option<Instant>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static SHELL_MSG: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static TASKBAR_MSG: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static RELAYOUT_PENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Mượn App an toàn: nếu đang được mượn (gọi lồng), trả về None thay vì panic.
fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| match a.try_borrow_mut() {
        Ok(mut g) => g.as_mut().map(f),
        Err(_) => None,
    })
}

fn lo(lp: LPARAM) -> i32 {
    (lp & 0xFFFF) as i16 as i32
}
fn hi(lp: LPARAM) -> i32 {
    ((lp >> 16) & 0xFFFF) as i16 as i32
}

fn short(s: &str, n: usize) -> String {
    let t: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{}…", t.trim_end())
    } else {
        t
    }
}

impl App {
    fn s(&self, v: i32) -> i32 {
        (v as f32 * self.dpi as f32 / 96.0).round() as i32
    }
    fn sc(&self) -> f32 {
        self.dpi as f32 / 96.0
    }
    fn sidebar_w(&self) -> i32 {
        if self.cfg.sidebar_collapsed {
            self.s(72)
        } else {
            self.s(248)
        }
    }

    fn toggle_sidebar(&mut self) {
        self.cfg.sidebar_collapsed = !self.cfg.sidebar_collapsed;
        self.cfg.save();
        self.hover = Hit::None;
        hide_tip(self.tip);
        self.layout();
    }

    /// Chú thích khi rê chuột (thanh thu gọn chỉ có biểu tượng)
    fn tip_for(&self, h: Hit) -> Option<(String, RECT)> {
        let r = self.hits.iter().rev().find(|(_, x)| *x == h).map(|(r, _)| *r)?;
        let collapsed = self.cfg.sidebar_collapsed;
        let t = match h {
            Hit::Account(i) if collapsed => {
                let a = self.cfg.accounts.get(i)?;
                let (st, _, _) = self.status_of(&a.id);
                format!("{} — {}", a.name, st)
            }
            Hit::Btn(Act::AddAcc) if collapsed => "Thêm tài khoản".into(),
            Hit::Btn(Act::Toggle) => {
                if collapsed {
                    "Mở rộng thanh tài khoản".into()
                } else {
                    "Thu gọn thanh tài khoản".into()
                }
            }
            _ => return None,
        };
        Some((t, r))
    }
    fn header_h(&self) -> i32 {
        self.s(56)
    }

    fn selected_index(&self) -> Option<usize> {
        self.cfg.selected.as_ref().and_then(|id| self.cfg.index_of(id))
    }
    fn selected_id(&self) -> Option<String> {
        self.selected_index().map(|i| self.cfg.accounts[i].id.clone())
    }
    fn account(&self, id: &str) -> Option<&Account> {
        self.cfg.accounts.iter().find(|a| a.id == id)
    }

    fn invalidate(&self) {
        unsafe {
            InvalidateRect(self.hwnd, null(), 0);
            InvalidateRect(self.pane, null(), 0);
        }
    }

    fn app_active(&self) -> bool {
        unsafe {
            if self.hidden_to_tray || IsWindowVisible(self.hwnd) == 0 || IsIconic(self.hwnd) != 0 {
                return false;
            }
            let fg = GetForegroundWindow();
            if fg == self.hwnd {
                return true;
            }
            if let Some(id) = self.selected_id() {
                if let Some(s) = self.sessions.get(&id) {
                    let mut pid = 0u32;
                    GetWindowThreadProcessId(fg, &mut pid);
                    return s.pids().contains(&pid);
                }
            }
            false
        }
    }

    /// cửa sổ Zalo đang chọn (chỉ khi nhúng)
    fn embedded_selected(&self) -> HWND {
        if !self.cfg.embed {
            return null_mut();
        }
        self.selected_id().and_then(|id| self.sessions.get(&id).map(|s| s.hwnd)).unwrap_or(null_mut())
    }

    // ---------------- bố cục ----------------
    fn layout(&mut self) {
        unsafe {
            let mut cr: RECT = std::mem::zeroed();
            GetClientRect(self.hwnd, &mut cr);
            let (sw, hh) = (self.sidebar_w(), self.header_h());
            MoveWindow(self.pane, sw, hh, (cr.right - sw).max(0), (cr.bottom - hh).max(0), 1);
        }
        self.place_all();
        self.invalidate();
    }

    fn place_all(&mut self) {
        let sel = self.selected_id();
        let embed = self.cfg.embed;
        let (pane, main) = (self.pane, self.hwnd);
        for (id, s) in self.sessions.iter_mut() {
            if s.hwnd.is_null() {
                continue;
            }
            if Some(id) == sel.as_ref() {
                place(s, embed, pane, main);
            } else {
                park(s, embed, pane);
            }
        }
    }

    // ---------------- phiên ----------------
    fn start(&mut self, id: &str) {
        if self.sessions.contains_key(id) || self.cfg.index_of(id).is_none() {
            return;
        }
        if let Some(st) = self.stopping.get_mut(id) {
            st.restart = true;
            self.invalidate();
            return;
        }
        if !self.zalo.as_ref().is_some_and(|p| p.is_file()) {
            self.zalo = session::find_zalo(self.cfg.zalo_path.as_deref());
        }
        let Some(exe) = self.zalo.clone() else {
            self.invalidate();
            return;
        };
        self.failed.remove(id);
        let profile = util::profiles_dir().join(id);
        match session::start(&exe, &profile) {
            Ok(s) => {
                self.sessions.insert(id.to_string(), s);
            }
            Err(e) => {
                log(&format!("Lỗi mở {id}: {e}"));
                self.failed.insert(id.to_string(), e);
            }
        }
        self.update_tray_tip();
        self.invalidate();
    }

    fn stop(&mut self, id: &str) {
        self.stop_with(id, false);
    }

    fn restart(&mut self, id: &str) {
        self.stop_with(id, true);
    }

    fn stop_with(&mut self, id: &str, restart: bool) {
        self.queue.retain(|x| x != id);
        if let Some(s) = self.sessions.remove(id) {
            unsafe {
                if !s.hwnd.is_null() {
                    ShowWindowAsync(s.hwnd, SW_HIDE);
                }
            }
            session::terminate(s.job);
            self.stopping.insert(id.to_string(), Stopping { job: s.job, since: Instant::now(), restart });
            log(&format!("Đang tắt phiên {id}"));
        } else if let Some(st) = self.stopping.get_mut(id) {
            st.restart = restart;
        }
        self.failed.remove(id);
        self.clear_unseen(id);
        self.update_tray_tip();
        self.invalidate();
        if restart && !self.stopping.contains_key(id) {
            self.start(id);
        }
    }

    fn reap_stopping(&mut self) {
        let mut restart_ids = vec![];
        let mut timed_out = vec![];
        self.stopping.retain(|id, st| {
            let gone = session::job_pids(st.job).is_empty();
            if !gone && st.since.elapsed() < STOP_TIMEOUT {
                return true;
            }
            unsafe { CloseHandle(st.job) };
            if gone {
                log(&format!("Đã tắt phiên {id}"));
                if st.restart {
                    restart_ids.push(id.clone());
                }
            } else {
                log(&format!("Phiên {id} chưa thoát sau {}s", STOP_TIMEOUT.as_secs()));
                timed_out.push(id.clone());
            }
            false
        });
        for id in timed_out {
            self.failed.insert(id, "Phiên cũ chưa thoát hẳn, hãy thử lại.".into());
        }
        for id in restart_ids {
            self.start(&id);
        }
    }

    fn clear_unseen(&mut self, id: &str) {
        self.has_new.remove(id);
        self.pending.remove(id);
        self.flash_pending.remove(id);
    }

    fn apply_dpi(&mut self, dpi: u32) {
        if self.dpi != dpi {
            self.dpi = dpi;
            self.fonts.free();
            self.fonts = Fonts::new(self.sc());
        }
    }

    fn reattach(&mut self, id: &str) {
        let embed = self.cfg.embed;
        let (pane, main) = (self.pane, self.hwnd);
        let is_sel = self.selected_id().as_deref() == Some(id);
        if let Some(s) = self.sessions.get_mut(id) {
            if !s.hwnd.is_null() && unsafe { IsWindow(s.hwnd) } != 0 {
                release(s);
            }
            s.hwnd = null_mut();
            let pids = s.pids();
            if let Some(wi) = session::best_window(&pids, true, null_mut()) {
                unsafe {
                    if IsWindowVisible(wi.hwnd) == 0 {
                        ShowWindowAsync(wi.hwnd, SW_SHOW);
                    }
                }
                adopt(s, &wi, embed, pane, main, is_sel);
            } else {
                log(&format!("Gắn lại: không tìm thấy cửa sổ nào của {id} (pids {:?})", pids));
            }
        }
        self.invalidate();
    }

    fn select(&mut self, idx: usize) {
        let Some(acc) = self.cfg.accounts.get(idx) else { return };
        let id = acc.id.clone();
        if self.cfg.selected.as_deref() != Some(id.as_str()) {
            let prev = self.embedded_selected();
            deactivate_child(prev);
        }
        self.cfg.selected = Some(id.clone());
        self.save_due.get_or_insert_with(Instant::now);
        self.has_new.remove(&id);
        self.pending.remove(&id);
        // tài khoản đang tắt thì chỉ chọn, người dùng tự bấm "Mở Zalo" khi cần
        self.place_all();
        self.update_tray_tip();
        self.invalidate();
        unsafe { PostMessageW(self.hwnd, WM_APP_FOCUS, 0, 0) };
    }

    fn select_id(&mut self, id: &str) {
        if let Some(i) = self.cfg.index_of(id) {
            // cuộn để thấy tài khoản
            if i < self.scroll {
                self.scroll = i;
            }
            self.select(i);
        }
    }

    fn set_mode(&mut self, embed: bool) {
        if self.cfg.embed == embed {
            return;
        }
        self.cfg.embed = embed;
        self.cfg.save();
        let (pane, main) = (self.pane, self.hwnd);
        for s in self.sessions.values_mut() {
            if s.hwnd.is_null() || unsafe { IsWindow(s.hwnd) } == 0 {
                continue;
            }
            release(s);
            apply_mode(s, embed, pane, main);
        }
        self.place_all();
        log(&format!("Đổi chế độ hiển thị: {}", if embed { "nhúng" } else { "gắn kèm" }));
        self.invalidate();
    }

    // ---------------- tin mới / thông báo ----------------
    fn new_message(&mut self, id: &str, title: &str, text: &str) {
        let active = self.selected_id().as_deref() == Some(id) && self.app_active();
        if active {
            return;
        }
        let Some(acc) = self.account(id).cloned() else { return };
        let first = self.has_new.insert(id.to_string());
        self.flash_pending.remove(id);
        log(&format!(
            "Tin mới ở {} ({})",
            acc.name,
            if title.is_empty() { "không rõ hội thoại" } else { "có nội dung" }
        ));
        if first {
            self.invalidate();
        }
        self.update_tray_tip();
        if !self.cfg.notify || acc.muted {
            return;
        }
        let recent = self.last_notify.get(id).map(|t| t.elapsed() < NOTIFY_COOLDOWN).unwrap_or(false);
        if recent {
            let p = self.pending.entry(id.to_string()).or_insert(Pending {
                count: 0,
                title: String::new(),
                text: String::new(),
            });
            p.count += 1;
            p.title = title.to_string();
            p.text = text.to_string();
        } else {
            self.send_notification(id, 1, title, text);
        }
    }

    fn send_notification(&mut self, id: &str, count: u32, title: &str, text: &str) {
        let Some(acc) = self.account(id).cloned() else { return };
        let (t, body) = if count > 1 {
            (acc.name.clone(), format!("{count} tin nhắn mới"))
        } else if self.cfg.hide_content || text.is_empty() {
            (acc.name.clone(), "Có tin nhắn mới".to_string())
        } else if title.is_empty() {
            (acc.name.clone(), short(text, 200))
        } else {
            (format!("{} · {}", acc.name, short(title, 40)), short(text, 200))
        };
        tray::notify(self.hwnd, &t, &body);
        self.last_notify.insert(id.to_string(), Instant::now());
        self.last_notified_id = Some(id.to_string());
        unsafe {
            if !self.hidden_to_tray && !self.app_active() {
                let fi = FLASHWINFO {
                    cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                    hwnd: self.hwnd,
                    dwFlags: FLASHW_TRAY | FLASHW_TIMERNOFG,
                    uCount: 3,
                    dwTimeout: 0,
                };
                FlashWindowEx(&fi);
            }
        }
    }

    fn update_tray_tip(&self) {
        let n = self.sessions.len();
        let tip = if !self.has_new.is_empty() {
            let names: Vec<String> =
                self.cfg.accounts.iter().filter(|a| self.has_new.contains(&a.id)).map(|a| a.name.clone()).collect();
            format!("{APP_NAME} – tin mới: {}", names.join(", "))
        } else if n == 0 {
            APP_NAME.to_string()
        } else {
            format!("{APP_NAME} – {n} tài khoản đang chạy")
        };
        tray::set_tip(self.hwnd, &short(&tip, 120));
    }

    fn show_main(&mut self) {
        unsafe {
            self.hidden_to_tray = false;
            ShowWindow(self.hwnd, SW_SHOW);
            if IsIconic(self.hwnd) != 0 {
                ShowWindow(self.hwnd, SW_RESTORE);
            }
            SetForegroundWindow(self.hwnd);
        }
        self.place_all();
        self.invalidate();
        unsafe { PostMessageW(self.hwnd, WM_APP_FOCUS, 0, 0) };
    }

    fn tick(&mut self) {
        self.reap_stopping();
        if self.save_due.is_some_and(|t| t.elapsed() >= SAVE_DELAY) {
            self.save_due = None;
            self.cfg.save();
        }
        let sel = self.selected_id();
        let embed = self.cfg.embed;
        let (pane, main) = (self.pane, self.hwnd);
        let mut switch_to: Option<String> = None;
        let mut dead: Vec<(String, Option<String>)> = vec![];

        for (id, s) in self.sessions.iter_mut() {
            let is_sel = Some(id) == sel.as_ref();
            let pids = s.pids();
            if pids.is_empty() {
                let quick = !s.ever_window && s.started.elapsed().as_secs() < 20;
                let why = if quick {
                    Some("Zalo đã tự thoát ngay sau khi mở. Có thể Zalo không cho chạy nhiều phiên cùng lúc trên máy này. Bạn thử bấm \"Thử lại\"; nếu vẫn lỗi, hãy cập nhật Zalo PC lên bản mới nhất.".to_string())
                } else {
                    None
                };
                log(&format!(
                    "Phiên {id} đã thoát (sau {}s, từng có cửa sổ: {})",
                    s.started.elapsed().as_secs(),
                    s.ever_window
                ));
                dead.push((id.clone(), why));
                continue;
            }
            unsafe {
                if !s.hwnd.is_null() && IsHungAppWindow(s.hwnd) != 0 {
                    continue;
                }
                if !s.hwnd.is_null() && IsWindow(s.hwnd) == 0 {
                    log(&format!("Cửa sổ của {id} đã đóng, tìm cửa sổ mới"));
                    s.hwnd = null_mut();
                    s.mode = None;
                }
                if s.hwnd.is_null() {
                    if let Some(wi) = session::best_window(&pids, false, null_mut()) {
                        adopt(s, &wi, embed, pane, main, is_sel);
                    }
                } else if is_sel {
                    if IsWindowVisible(s.hwnd) == 0 && IsWindowVisible(main) != 0 && IsIconic(main) == 0 {
                        s.hidden_ticks += 1;
                        if let Some(wi) = session::best_window(&pids, false, s.hwnd) {
                            log(&format!("{id}: cửa sổ chính bị ẩn, chuyển sang cửa sổ mới class='{}'", wi.class));
                            release(s);
                            adopt(s, &wi, embed, pane, main, true);
                        } else if s.hidden_ticks >= 5 {
                            s.hidden_ticks = 0;
                            place(s, embed, pane, main);
                        }
                    } else {
                        s.hidden_ticks = 0;
                        if IsIconic(s.hwnd) != 0 || IsZoomed(s.hwnd) != 0 {
                            ShowWindow(s.hwnd, SW_RESTORE);
                            place(s, embed, pane, main);
                        } else if !embed
                            && IsIconic(main) == 0
                            && IsWindowVisible(main) != 0
                            && GetAsyncKeyState(VK_LBUTTON as i32) >= 0
                        {
                            // chế độ gắn kèm: nếu cửa sổ Zalo bị kéo lệch thì đưa về đúng chỗ
                            let mut a: RECT = std::mem::zeroed();
                            let mut b: RECT = std::mem::zeroed();
                            GetWindowRect(s.hwnd, &mut a);
                            GetWindowRect(pane, &mut b);
                            if a.left != b.left || a.top != b.top || a.right != b.right || a.bottom != b.bottom {
                                place(s, embed, pane, main);
                            }
                        }
                    }
                } else if embed {
                    // tài khoản không chọn: giữ cửa sổ "đang hiển thị" nhưng nằm ngoài khung nhìn,
                    // để Zalo vẫn cập nhật giao diện (giúp phát hiện tin mới)
                    if IsWindowVisible(s.hwnd) == 0 && IsWindowVisible(main) != 0 {
                        park(s, embed, pane);
                    }
                } else if IsWindowVisible(s.hwnd) != 0 {
                    // Zalo tự hiện cửa sổ (bấm biểu tượng khay / thông báo của Zalo) -> chuyển sang tab đó
                    log(&format!("{id}: Zalo tự hiện cửa sổ, chuyển tab"));
                    switch_to = Some(id.clone());
                }
                if !s.hwnd.is_null() {
                    s.unread = session::parse_unread(&session::window_title(s.hwnd));
                }
            }
        }
        for (id, why) in dead {
            if let Some(s) = self.sessions.remove(&id) {
                unsafe { CloseHandle(s.job) };
            }
            self.clear_unseen(&id);
            if let Some(w) = why {
                self.failed.insert(id, w);
            }
            self.update_tray_tip();
        }
        if let Some(id) = switch_to {
            self.select_id(&id);
        }

        // tài khoản đang xem: xóa dấu tin mới
        let active = self.app_active();
        if active {
            if let Some(id) = self.selected_id() {
                if self.has_new.remove(&id) {
                    self.pending.remove(&id);
                    self.update_tray_tip();
                    self.invalidate();
                }
            }
        }

        // gửi các thông báo đang chờ (gom tin dồn dập)
        let ready: Vec<String> = self
            .pending
            .keys()
            .filter(|id| self.last_notify.get(*id).map(|t| t.elapsed() >= NOTIFY_COOLDOWN).unwrap_or(true))
            .cloned()
            .collect();
        for id in ready {
            if let Some(p) = self.pending.remove(&id) {
                if self.has_new.contains(&id) {
                    self.send_notification(&id, p.count, &p.title, &p.text);
                }
            }
        }
        // tín hiệu "nhấp nháy" của Zalo mà chưa đọc được nội dung -> báo chung
        let flashed: Vec<String> = self
            .flash_pending
            .iter()
            .filter(|(_, t)| t.elapsed() > Duration::from_secs(3))
            .map(|(id, _)| id.clone())
            .collect();
        for id in flashed {
            self.flash_pending.remove(&id);
            self.new_message(&id, "", "");
        }

        // cập nhật danh sách cho luồng theo dõi tin mới
        let targets: Vec<watcher::Target> = self
            .sessions
            .iter()
            .filter(|(_, s)| !s.hwnd.is_null())
            .map(|(id, s)| watcher::Target {
                id: id.clone(),
                hwnd: s.hwnd as isize,
                active: active && Some(id) == sel.as_ref(),
            })
            .collect();
        watcher::set_targets(targets);

        // tự mở lần lượt các tài khoản khi khởi động
        if !self.queue.is_empty() && self.last_auto.elapsed().as_millis() > 1500 {
            if let Some(id) = self.queue.pop_front() {
                if self.cfg.index_of(&id).is_some() {
                    self.start(&id);
                }
                self.last_auto = Instant::now();
            }
        }
        // vẽ lại khi trạng thái đổi
        let mut sig = String::new();
        for a in &self.cfg.accounts {
            let st = match self.sessions.get(&a.id) {
                Some(s) => format!("{:?}{}{}", s.state, s.hwnd.is_null(), s.unread),
                None => format!("off{}{}", self.failed.contains_key(&a.id), self.stopping.contains_key(&a.id)),
            };
            sig.push_str(&st);
            if let Some(s) = self.sessions.get(&a.id) {
                if s.hwnd.is_null() && s.started.elapsed().as_secs() > 20 {
                    sig.push('!');
                }
            }
        }
        if sig != self.sig {
            self.sig = sig;
            self.invalidate();
        }
    }

    fn on_flash(&mut self, h: HWND) {
        if h == self.hwnd || h.is_null() {
            return;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(h, &mut pid) };
        let id = self.sessions.iter().find(|(_, s)| s.hwnd == h || s.pids().contains(&pid)).map(|(id, _)| id.clone());
        if let Some(id) = id {
            let active = self.selected_id().as_deref() == Some(id.as_str()) && self.app_active();
            if !active && !self.has_new.contains(&id) {
                self.flash_pending.entry(id).or_insert_with(Instant::now);
            }
        }
    }
}

fn defer_layout(hwnd: HWND) {
    if !RELAYOUT_PENDING.with(|c| c.replace(true)) {
        unsafe { PostMessageW(hwnd, WM_APP_RELAYOUT, 0, 0) };
    }
}

fn request_focus() {
    let h = hwnd_main();
    if !h.is_null() {
        unsafe { PostMessageW(h, WM_APP_FOCUS, 0, 0) };
    }
}

// ---------------- window procedure ----------------

fn hit_at(hits: &[(RECT, Hit)], x: i32, y: i32) -> Hit {
    hits.iter().rev().find(|(r, _)| contains(r, x, y)).map(|(_, h)| *h).unwrap_or(Hit::None)
}

unsafe fn double_buffer(hwnd: HWND, f: impl FnOnce(HDC)) {
    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut cr: RECT = std::mem::zeroed();
    GetClientRect(hwnd, &mut cr);
    let mdc = CreateCompatibleDC(hdc);
    let bmp = CreateCompatibleBitmap(hdc, cr.right.max(1), cr.bottom.max(1));
    let old = SelectObject(mdc, bmp as _);
    f(mdc);
    BitBlt(hdc, 0, 0, cr.right, cr.bottom, mdc, 0, 0, SRCCOPY);
    SelectObject(mdc, old);
    DeleteObject(bmp as _);
    DeleteDC(mdc);
    EndPaint(hwnd, &ps);
}

fn track_leave(hwnd: HWND) {
    let mut t = TRACKMOUSEEVENT {
        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };
    unsafe { TrackMouseEvent(&mut t) };
}

unsafe extern "system" fn main_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // thông điệp đăng ký động
    let shell = SHELL_MSG.with(|c| c.get());
    if shell != 0 && msg == shell {
        if (wp & 0xFFFF) == HSHELL_FLASH_X {
            with_app(|a| a.on_flash(lp as HWND));
        }
        return 0;
    }
    let tb = TASKBAR_MSG.with(|c| c.get());
    if tb != 0 && msg == tb {
        // Explorer khởi động lại: thêm lại biểu tượng khay
        tray::add(hwnd, APP_NAME);
        with_app(|a| a.update_tray_tip());
        return 0;
    }
    match msg {
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let mut done = false;
            double_buffer(hwnd, |hdc| {
                done = with_app(|a| a.paint_main(hdc)).is_some();
            });
            if !done {
                InvalidateRect(hwnd, null(), 0);
            }
            0
        }
        WM_SIZE => {
            if wp as u32 != SIZE_MINIMIZED && with_app(|a| a.layout()).is_none() {
                defer_layout(hwnd);
            }
            0
        }
        WM_MOVE | WM_WINDOWPOSCHANGED => {
            let done = with_app(|a| {
                if !a.cfg.embed {
                    a.place_all();
                }
            });
            if done.is_none() {
                defer_layout(hwnd);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_APP_RELAYOUT => {
            RELAYOUT_PENDING.with(|c| c.set(false));
            if IsIconic(hwnd) == 0 {
                let dpi = GetDpiForWindow(hwnd).max(96);
                if with_app(|a| {
                    a.apply_dpi(dpi);
                    a.layout();
                })
                .is_none()
                {
                    defer_layout(hwnd);
                }
            }
            0
        }
        WM_SHOWWINDOW => {
            if wp != 0 {
                with_app(|a| a.hidden_to_tray = false);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_GETMINMAXINFO => {
            let mm = &mut *(lp as *mut MINMAXINFO);
            let dpi = GetDpiForWindow(hwnd).max(96) as i32;
            mm.ptMinTrackSize.x = 860 * dpi / 96;
            mm.ptMinTrackSize.y = 600 * dpi / 96;
            0
        }
        WM_DPICHANGED => {
            let r = &*(lp as *const RECT);
            if with_app(|a| a.apply_dpi((wp & 0xFFFF) as u32)).is_none() {
                defer_layout(hwnd);
            }
            SetWindowPos(
                hwnd,
                null_mut(),
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            0
        }
        WM_MOUSEMOVE => {
            track_leave(hwnd);
            with_app(|a| {
                let h = hit_at(&a.hits, lo(lp), hi(lp));
                if h != a.hover {
                    a.hover = h;
                    InvalidateRect(hwnd, null(), 0);
                    match a.tip_for(h) {
                        Some((t, r)) => {
                            let mut pt = POINT { x: a.sidebar_w() + a.s(6), y: (r.top + r.bottom) / 2 - a.s(12) };
                            if h == Hit::Btn(Act::Toggle) && !a.cfg.sidebar_collapsed {
                                pt = POINT { x: r.right + a.s(6), y: r.top };
                            }
                            ClientToScreen(hwnd, &mut pt);
                            show_tip(a.tip, &t, pt.x, pt.y);
                        }
                        None => hide_tip(a.tip),
                    }
                }
            });
            0
        }
        WM_MOUSELEAVE_X => {
            with_app(|a| {
                hide_tip(a.tip);
                if a.hover != Hit::None {
                    a.hover = Hit::None;
                    InvalidateRect(hwnd, null(), 0);
                }
            });
            0
        }
        WM_MOUSEWHEEL => {
            let delta = ((wp >> 16) & 0xFFFF) as i16 as i32;
            let mut pt = POINT { x: lo(lp), y: hi(lp) };
            ScreenToClient(hwnd, &mut pt);
            with_app(|a| {
                if contains(&a.list_rect, pt.x, pt.y) {
                    if delta > 0 {
                        a.scroll = a.scroll.saturating_sub(1);
                    } else {
                        a.scroll += 1;
                    }
                    a.invalidate();
                }
            });
            0
        }
        WM_SETCURSOR => {
            if (lp & 0xFFFF) as u32 == HTCLIENT
                && wp as HWND == hwnd
                && with_app(|a| a.hover != Hit::None).unwrap_or(false)
            {
                SetCursor(LoadCursorW(null_mut(), IDC_HAND));
                return 1;
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_LBUTTONUP | WM_RBUTTONUP => {
            let (x, y) = (lo(lp), hi(lp));
            let h = with_app(|a| {
                hide_tip(a.tip);
                hit_at(&a.hits, x, y)
            })
            .unwrap_or(Hit::None);
            let mut pt = POINT { x, y };
            ClientToScreen(hwnd, &mut pt);
            if msg == WM_RBUTTONUP {
                if let Hit::Account(i) = h {
                    account_menu(i, pt.x, pt.y);
                }
            } else {
                if h == Hit::Btn(Act::Settings) {
                    if let Some(r) = with_app(|a| a.hits.iter().find(|(_, hh)| *hh == h).map(|(r, _)| *r)).flatten() {
                        let mut p2 = POINT { x: r.left, y: r.bottom + 4 };
                        ClientToScreen(hwnd, &mut p2);
                        pt = p2;
                    }
                }
                do_hit(h, pt);
            }
            0
        }
        WM_ACTIVATE => {
            if (wp & 0xFFFF) as u32 == WA_INACTIVE {
                if let Some(h) = with_app(|a| a.embedded_selected()) {
                    deactivate_child(h);
                }
            } else {
                request_focus();
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SETFOCUS => {
            request_focus();
            0
        }
        WM_APP_FOCUS => {
            if GetForegroundWindow() == hwnd && IsWindowEnabled(hwnd) != 0 {
                if let Some(h) = with_app(|a| a.embedded_selected()) {
                    activate_child(h);
                }
            }
            0
        }
        WM_TIMER => {
            if wp == TIMER_TICK {
                with_app(|a| a.tick());
            }
            0
        }
        tray::WM_TRAY => {
            let ev = (lp & 0xFFFF) as u32;
            match ev {
                0x0405 => {
                    // NIN_BALLOONUSERCLICK: bấm vào thông báo -> mở đúng tài khoản
                    with_app(|a| {
                        a.show_main();
                        if let Some(id) = a.last_notified_id.clone() {
                            a.select_id(&id);
                        }
                    });
                }
                WM_LBUTTONUP | 0x0400 | 0x0401 => {
                    with_app(|a| a.show_main());
                }
                WM_CONTEXTMENU | WM_RBUTTONUP => {
                    let x = (wp & 0xFFFF) as i16 as i32;
                    let y = ((wp >> 16) & 0xFFFF) as i16 as i32;
                    tray_menu(x, y);
                }
                _ => {}
            }
            0
        }
        watcher::WM_WATCH_EVENT => {
            for e in watcher::take_events() {
                with_app(|a| a.new_message(&e.id, &e.title, &e.text));
            }
            0
        }
        WM_CLOSE => {
            let to_tray = with_app(|a| a.cfg.close_to_tray).unwrap_or(false);
            if to_tray {
                let first = with_app(|a| {
                    a.hidden_to_tray = true;
                    let f = !a.cfg.tray_hint_shown;
                    a.cfg.tray_hint_shown = true;
                    a.cfg.save();
                    f
                })
                .unwrap_or(false);
                ShowWindow(hwnd, SW_HIDE);
                if first {
                    tray::notify(
                        hwnd,
                        &format!("{APP_NAME} vẫn đang chạy"),
                        "Bạn vẫn nhận được thông báo tin nhắn. Chuột phải biểu tượng Betacom ở khay hệ thống để thoát.",
                    );
                }
            } else {
                exit_app();
            }
            0
        }
        WM_APP_PURGE_FAILED => {
            report_purge_failures();
            0
        }
        WM_DESTROY => {
            with_app(|a| {
                if a.save_due.take().is_some() {
                    a.cfg.save();
                }
            });
            tray::remove(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe extern "system" fn pane_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_PARENTNOTIFY => {
            // người dùng bấm chuột vào cửa sổ Zalo nhúng -> trao bàn phím cho Zalo
            let ev = (wp & 0xFFFF) as u32;
            if ev == WM_LBUTTONDOWN || ev == WM_RBUTTONDOWN || ev == WM_MBUTTONDOWN || ev == 0x0246 {
                request_focus();
            }
            0
        }
        WM_MOUSEACTIVATE => {
            request_focus();
            MA_ACTIVATE as LRESULT
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let mut done = false;
            double_buffer(hwnd, |hdc| {
                done = with_app(|a| a.paint_pane(hdc)).is_some();
            });
            if !done {
                InvalidateRect(hwnd, null(), 0);
            }
            0
        }
        WM_MOUSEMOVE => {
            track_leave(hwnd);
            with_app(|a| {
                let h = hit_at(&a.pane_hits, lo(lp), hi(lp));
                if h != a.pane_hover {
                    a.pane_hover = h;
                    InvalidateRect(hwnd, null(), 0);
                }
            });
            0
        }
        WM_MOUSELEAVE_X => {
            with_app(|a| {
                if a.pane_hover != Hit::None {
                    a.pane_hover = Hit::None;
                    InvalidateRect(hwnd, null(), 0);
                }
            });
            0
        }
        WM_SETCURSOR => {
            if (lp & 0xFFFF) as u32 == HTCLIENT
                && wp as HWND == hwnd
                && with_app(|a| a.pane_hover != Hit::None).unwrap_or(false)
            {
                SetCursor(LoadCursorW(null_mut(), IDC_HAND));
                return 1;
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_LBUTTONUP => {
            let (x, y) = (lo(lp), hi(lp));
            let h = with_app(|a| hit_at(&a.pane_hits, x, y)).unwrap_or(Hit::None);
            let mut pt = POINT { x, y };
            ClientToScreen(hwnd, &mut pt);
            do_hit(h, pt);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn legacy_running() -> bool {
    unsafe {
        let m = w("Local\\SudoSoftware.MultiZ.SingleInstance");
        let h = OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, m.as_ptr());
        if !h.is_null() {
            CloseHandle(h);
            return true;
        }
        let c = w("MultiZMain");
        !FindWindowW(c.as_ptr(), null()).is_null()
    }
}

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let start_in_tray = std::env::args().any(|a| a == "--tray");

        let mname = w("Local\\Betacom.BetaZalo.SingleInstance");
        CreateMutexW(null(), 0, mname.as_ptr());
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let c = w(MAIN_CLASS);
            let h = FindWindowW(c.as_ptr(), null());
            if !h.is_null() {
                ShowWindow(h, SW_SHOW);
                ShowWindow(h, SW_RESTORE);
                SetForegroundWindow(h);
            }
            return;
        }
        if legacy_running() {
            let t = w(&format!(
                "Bản MultiZ cũ đang chạy. Hãy thoát MultiZ (biểu tượng dưới khay hệ thống) rồi mở lại {APP_NAME} để dữ liệu được chuyển sang an toàn."
            ));
            let c = w(APP_NAME);
            MessageBoxW(null_mut(), t.as_ptr(), c.as_ptr(), MB_OK | MB_ICONWARNING);
            return;
        }

        log(&format!("=== {APP_NAME} {APP_VERSION} khởi động ==="));
        if let Some(note) = util::migration_note() {
            log(note);
        }
        migrate_autorun();
        gdiplus_init();
        let hinst = GetModuleHandleW(null());
        let icon = LoadIconW(hinst, util::int_resource(1));

        let mc = w(MAIN_CLASS);
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfnWndProc = Some(main_proc);
        wc.hInstance = hinst;
        wc.hIcon = icon;
        wc.hIconSm = icon;
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc.lpszClassName = mc.as_ptr();
        RegisterClassExW(&wc);

        let pc = w(PANE_CLASS);
        let mut wc2: WNDCLASSEXW = std::mem::zeroed();
        wc2.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc2.lpfnWndProc = Some(pane_proc);
        wc2.hInstance = hinst;
        wc2.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc2.lpszClassName = pc.as_ptr();
        RegisterClassExW(&wc2);

        let sdpi = GetDpiForSystem().max(96) as i32;
        let (mut ww, mut wh) = (1280 * sdpi / 96, 820 * sdpi / 96);
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        ww = ww.min(sw - 40);
        wh = wh.min(sh - 80);
        let title = w(&format!("{APP_NAME} – nhiều tài khoản Zalo"));
        let hwnd = CreateWindowExW(
            0,
            mc.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            (sw - ww) / 2,
            (sh - wh) / 2,
            ww,
            wh,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        let pane = CreateWindowExW(
            0,
            pc.as_ptr(),
            null(),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            0,
            0,
            10,
            10,
            hwnd,
            null_mut(),
            hinst,
            null(),
        );

        let mut cfg = Config::load();
        let zalo = session::find_zalo(cfg.zalo_path.as_deref());
        log(&format!("Zalo.exe: {:?}", zalo));
        let dpi = GetDpiForWindow(hwnd).max(96);

        // hàng đợi tự mở: tài khoản đang chọn trước
        let mut queue: VecDeque<String> = VecDeque::new();
        if zalo.is_some() {
            if let Some(sel) = &cfg.selected {
                if cfg.accounts.iter().any(|a| &a.id == sel && a.autostart) {
                    queue.push_back(sel.clone());
                }
            }
            for a in &cfg.accounts {
                if a.autostart && !queue.contains(&a.id) {
                    queue.push_back(a.id.clone());
                }
            }
        }
        if cfg.selected.as_ref().map(|s| cfg.index_of(s).is_none()).unwrap_or(true) {
            cfg.selected = cfg.accounts.first().map(|a| a.id.clone());
        }
        let avatars = avatar::load_all(&cfg);

        let app = App {
            hwnd,
            pane,
            cfg,
            sessions: HashMap::new(),
            stopping: HashMap::new(),
            failed: HashMap::new(),
            zalo,
            dpi,
            fonts: Fonts::new(dpi as f32 / 96.0),
            hits: vec![],
            pane_hits: vec![],
            hover: Hit::None,
            pane_hover: Hit::None,
            queue,
            last_auto: Instant::now() - Duration::from_secs(10),
            sig: String::new(),
            has_new: HashSet::new(),
            last_notify: HashMap::new(),
            pending: HashMap::new(),
            flash_pending: HashMap::new(),
            last_notified_id: None,
            hidden_to_tray: start_in_tray,
            avatars,
            scroll: 0,
            list_rect: rc(0, 0, 0, 0),
            tip: create_tip(hwnd),
            save_due: None,
        };
        MAIN_HWND.store(hwnd as isize, std::sync::atomic::Ordering::Relaxed);
        let read_only = app.cfg.read_only;
        if !read_only {
            sweep_trash();
        }
        APP.with(|a| *a.borrow_mut() = Some(app));

        // khay hệ thống + bắt tín hiệu nhấp nháy của cửa sổ
        tray::add(hwnd, APP_NAME);
        let shell_name = w("SHELLHOOK");
        SHELL_MSG.with(|c| c.set(RegisterWindowMessageW(shell_name.as_ptr())));
        RegisterShellHookWindow(hwnd);
        let tb_name = w("TaskbarCreated");
        TASKBAR_MSG.with(|c| c.set(RegisterWindowMessageW(tb_name.as_ptr())));

        with_app(|a| a.layout());
        if !start_in_tray {
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
        }
        SetTimer(hwnd, TIMER_TICK, 400, None);
        watcher::start(hwnd as isize);
        if read_only {
            msgbox(
                &format!(
                    "Không đọc được danh sách tài khoản:\n{}\n\n{APP_NAME} đang chạy ở chế độ không lưu: thay đổi sẽ không được ghi và dữ liệu tài khoản không bị xóa. Hãy đóng chương trình đang khóa file này rồi mở lại {APP_NAME}.",
                    util::data_root().join("config.json").display()
                ),
                MB_OK | MB_ICONWARNING,
            );
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        log(&format!("{APP_NAME} đã thoát"));
    }
}
