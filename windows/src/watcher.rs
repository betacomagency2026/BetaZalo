//! Theo dõi tin nhắn mới của từng phiên Zalo bằng UI Automation (chạy ở luồng riêng).
//!
//! Cách làm: đọc các dòng chữ trong cột danh sách hội thoại (bên trái cửa sổ Zalo),
//! chuẩn hóa (bỏ giờ, số) rồi so với lần đọc trước. Dòng mới xuất hiện = có tin mới.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use windows::core::VARIANT;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::Com::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsWindow, PostMessageW};

use crate::util::log;

pub const WM_WATCH_EVENT: u32 = 0x8000 + 11; // WM_APP + 11

#[derive(Clone)]
pub struct Target {
    pub id: String,
    pub hwnd: isize,
    /// người dùng đang xem tài khoản này (không cần báo)
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub text: String,
}

#[derive(Default)]
struct Shared {
    targets: Vec<Target>,
    events: Vec<Event>,
}

fn shared() -> &'static Mutex<Shared> {
    static S: OnceLock<Mutex<Shared>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Shared::default()))
}

pub fn set_targets(t: Vec<Target>) {
    if let Ok(mut s) = shared().lock() {
        s.targets = t;
    }
}

pub fn take_events() -> Vec<Event> {
    shared().lock().map(|mut s| std::mem::take(&mut s.events)).unwrap_or_default()
}

struct Line {
    top: i32,
    left: i32,
    text: String,
    norm: String,
}

fn normalize(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut t = lower.clone();
    for wd in ["vừa xong", "hôm qua", "giây", "phút", "giờ", "ngày", "tuần", "tháng", "năm", "trước", "just now"]
    {
        t = t.replace(wd, " ");
    }
    let t: String =
        t.chars().map(|c| if c.is_ascii_digit() || c == '/' || c == ':' || c == '+' { ' ' } else { c }).collect();
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

unsafe fn read_lines(uia: &IUIAutomation, hwnd: isize) -> windows::core::Result<Vec<Line>> {
    let h = HWND(hwnd as *mut _);
    let el = uia.ElementFromHandle(h)?;
    let mut wr = RECT::default();
    let _ = GetWindowRect(h, &mut wr);
    let dpi = GetDpiForWindow(h).max(96) as f32 / 96.0;
    let width = (wr.right - wr.left) as f32;
    let x_min = (56.0 * dpi) as i32;
    let x_max = ((480.0 * dpi).min(width * 0.55)) as i32;
    let y_min = (56.0 * dpi) as i32;

    let cond = uia.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_TextControlTypeId.0))?;
    let cache = uia.CreateCacheRequest()?;
    cache.AddProperty(UIA_NamePropertyId)?;
    cache.AddProperty(UIA_BoundingRectanglePropertyId)?;
    // ưu tiên lấy kèm bộ đệm (nhanh); nếu hệ thống không hỗ trợ thì đọc từng phần tử
    let (arr, cached) = match el.FindAllBuildCache(TreeScope_Descendants, &cond, &cache) {
        Ok(a) => (a, true),
        Err(_) => (el.FindAll(TreeScope_Descendants, &cond)?, false),
    };
    let n = arr.Length()?;
    let mut out = vec![];
    for i in 0..n {
        let e = arr.GetElement(i)?;
        let name = if cached { e.CachedName() } else { e.CurrentName() };
        let name = name.map(|b| b.to_string()).unwrap_or_default();
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        let r = if cached { e.CachedBoundingRectangle() } else { e.CurrentBoundingRectangle() };
        let r = r.unwrap_or_default();
        if r.right <= r.left || r.bottom <= r.top {
            continue; // phần tử ẩn / ngoài màn hình
        }
        let cx = (r.left + r.right) / 2 - wr.left;
        let top = r.top - wr.top;
        if cx < x_min || cx > x_max || top < y_min || r.bottom > wr.bottom {
            continue;
        }
        let norm = normalize(&name);
        if norm.chars().count() < 2 {
            continue;
        }
        out.push(Line { top, left: r.left - wr.left, text: name, norm });
    }
    out.sort_by_key(|l| (l.top, l.left));
    Ok(out)
}

struct AccState {
    prev: HashSet<String>,
    seen_once: bool,
}

pub fn start(notify_hwnd: isize) {
    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let uia: IUIAutomation = match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
            Ok(u) => u,
            Err(e) => {
                log(&format!("Không khởi tạo được UI Automation: {e}"));
                return;
            }
        };
        log("Bắt đầu theo dõi tin nhắn mới (UI Automation)");
        let mut states: HashMap<String, AccState> = HashMap::new();
        let mut logged_empty: HashSet<String> = HashSet::new();
        let mut logged_err: HashSet<String> = HashSet::new();
        loop {
            std::thread::sleep(Duration::from_millis(2500));
            let targets = match shared().lock() {
                Ok(s) => s.targets.clone(),
                Err(_) => continue,
            };
            states.retain(|id, _| targets.iter().any(|t| &t.id == id));
            let mut new_events = vec![];
            for t in &targets {
                if !IsWindow(HWND(t.hwnd as *mut _)).as_bool() {
                    continue;
                }
                let started = Instant::now();
                let lines = match read_lines(&uia, t.hwnd) {
                    Ok(l) => l,
                    Err(e) => {
                        let msg = format!("{}: đọc giao diện lỗi {e}", t.id);
                        if logged_err.insert(msg.clone()) {
                            log(&msg);
                        }
                        continue;
                    }
                };
                if lines.is_empty() {
                    if logged_empty.insert(t.id.clone()) {
                        log(&format!("{}: chưa đọc được dòng chữ nào trong danh sách hội thoại (có thể đang ở màn hình đăng nhập)", t.id));
                    }
                    continue;
                }
                if logged_empty.remove(&t.id) || !states.contains_key(&t.id) {
                    log(&format!(
                        "{}: đọc được {} dòng trong danh sách hội thoại ({} ms)",
                        t.id,
                        lines.len(),
                        started.elapsed().as_millis()
                    ));
                }
                let cur: HashSet<String> = lines.iter().map(|l| l.norm.clone()).collect();
                let st = states.entry(t.id.clone()).or_insert(AccState { prev: HashSet::new(), seen_once: false });
                if !st.seen_once || t.active {
                    st.prev = cur;
                    st.seen_once = true;
                    continue;
                }
                let new_idx: Vec<usize> =
                    lines.iter().enumerate().filter(|(_, l)| !st.prev.contains(&l.norm)).map(|(i, _)| i).collect();
                // bỏ tin do chính mình gửi
                let real: Vec<usize> = new_idx
                    .iter()
                    .copied()
                    .filter(|&i| {
                        let t = lines[i].text.trim_start();
                        !(t.starts_with("Bạn:") || t.starts_with("Bạn :") || t.starts_with("You:"))
                    })
                    .collect();
                if !new_idx.is_empty() && new_idx.len() <= 4 && !real.is_empty() {
                    let i = real[0];
                    // tiêu đề = dòng ngay phía trên (tên hội thoại), nếu gần
                    let mut title = String::new();
                    let mut text = lines[i].text.clone();
                    if i > 0 && lines[i].top - lines[i - 1].top < 40 && lines[i].top - lines[i - 1].top > 4 {
                        title = lines[i - 1].text.clone();
                    } else if i + 1 < lines.len()
                        && lines[i + 1].top - lines[i].top < 40
                        && lines[i + 1].top - lines[i].top > 4
                    {
                        title = lines[i].text.clone();
                        text = lines[i + 1].text.clone();
                    }
                    new_events.push(Event { id: t.id.clone(), title, text });
                } else if new_idx.len() > 4 {
                    log(&format!("{}: {} dòng thay đổi cùng lúc, coi như tải lại danh sách", t.id, new_idx.len()));
                }
                st.prev = cur;
            }
            if !new_events.is_empty() {
                if let Ok(mut s) = shared().lock() {
                    s.events.extend(new_events);
                }
                let _ = PostMessageW(HWND(notify_hwnd as *mut _), WM_WATCH_EVENT, WPARAM(0), LPARAM(0));
            }
        }
    });
}
