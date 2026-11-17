//! Khởi chạy Zalo PC gốc trong một "vùng dữ liệu" riêng cho từng tài khoản.
//!
//! Cách tách: chạy Zalo.exe với các biến môi trường USERPROFILE / APPDATA /
//! LOCALAPPDATA / TEMP trỏ vào thư mục riêng của tài khoản. Mỗi phiên được
//! gom vào một Job Object để theo dõi toàn bộ tiến trình con và tắt sạch.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::time::Instant;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::JobObjects::*;
use windows_sys::Win32::System::Registry::*;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::util::{from_w, log, w};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Starting,
    Running,
}

pub struct Session {
    pub job: HANDLE,
    pub pid: u32,
    pub hwnd: HWND,
    /// chế độ đã áp dụng cho hwnd: Some(true)=nhúng, Some(false)=gắn kèm
    pub mode: Option<bool>,
    pub orig_style: isize,
    pub orig_exstyle: isize,
    pub state: State,
    pub started: Instant,
    pub hidden_ticks: u32,
    pub unread: u32,
    pub ever_window: bool,
}

impl Session {
    pub fn pids(&self) -> Vec<u32> {
        job_pids(self.job)
    }
}

pub fn job_pids(job: HANDLE) -> Vec<u32> {
    // header (2 x u32) + danh sách ULONG_PTR
    let mut buf = vec![0usize; 1 + 2048];
    let ok = unsafe {
        QueryInformationJobObject(
            job,
            JobObjectBasicProcessIdList,
            buf.as_mut_ptr() as *mut _,
            (buf.len() * std::mem::size_of::<usize>()) as u32,
            null_mut(),
        )
    };
    if ok == 0 {
        return vec![];
    }
    let hdr = buf.as_ptr() as *const u32;
    let n = unsafe { *hdr.add(1) } as usize;
    buf[1..1 + n.min(2048)].iter().map(|&p| p as u32).collect()
}

fn split_home(p: &Path) -> (String, String) {
    let s = p.to_string_lossy().to_string();
    if s.len() > 2 && s.as_bytes()[1] == b':' {
        (s[..2].to_string(), s[2..].to_string())
    } else {
        (String::new(), s)
    }
}

pub fn prepare_profile(profile: &Path) -> std::io::Result<()> {
    for sub in [
        "AppData\\Roaming",
        "AppData\\Local\\Temp",
        "AppData\\LocalLow",
        "Documents",
        "Downloads",
        "Desktop",
        "Pictures",
    ] {
        std::fs::create_dir_all(profile.join(sub))?;
    }
    Ok(())
}

pub fn start(exe: &Path, profile: &Path) -> Result<Session, String> {
    prepare_profile(profile).map_err(|e| format!("Không tạo được thư mục dữ liệu: {e}"))?;

    // --- môi trường riêng ---
    let mut env: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (k, v) in std::env::vars_os() {
        let k = k.to_string_lossy().to_string();
        if k.starts_with('=') {
            continue;
        }
        env.insert(k.to_uppercase(), (k, v.to_string_lossy().to_string()));
    }
    let p = |s: &str| profile.join(s).to_string_lossy().to_string();
    let (drive, homepath) = split_home(profile);
    let overrides = [
        ("USERPROFILE", profile.to_string_lossy().to_string()),
        ("APPDATA", p("AppData\\Roaming")),
        ("LOCALAPPDATA", p("AppData\\Local")),
        ("TEMP", p("AppData\\Local\\Temp")),
        ("TMP", p("AppData\\Local\\Temp")),
        ("HOMEDRIVE", drive),
        ("HOMEPATH", homepath),
        ("MULTIZALO_PROFILE", profile.to_string_lossy().to_string()),
    ];
    for (k, v) in overrides {
        env.insert(k.to_string(), (k.to_string(), v));
    }
    let mut block: Vec<u16> = Vec::new();
    for (k, v) in env.values() {
        block.extend(format!("{k}={v}").encode_utf16());
        block.push(0);
    }
    block.push(0);

    let mut cmd = w(&format!("\"{}\"", exe.display()));
    let cwd = exe.parent().map(|d| w(&d.to_string_lossy())).unwrap_or_else(|| w(""));

    unsafe {
        let job = CreateJobObjectW(null(), null());
        if job.is_null() {
            return Err("Không tạo được Job Object".into());
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );

        let mut si: STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        let ok = CreateProcessW(
            null(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            0,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
            block.as_ptr() as *const _,
            if exe.parent().is_some() { cwd.as_ptr() } else { null() },
            &si,
            &mut pi,
        );
        if ok == 0 {
            let err = GetLastError();
            CloseHandle(job);
            return Err(format!("Không mở được Zalo.exe (mã lỗi {err})"));
        }
        if AssignProcessToJobObject(job, pi.hProcess) == 0 {
            let err = GetLastError();
            log(&format!("AssignProcessToJobObject lỗi {err}"));
            TerminateProcess(pi.hProcess, 1);
            CloseHandle(pi.hThread);
            CloseHandle(pi.hProcess);
            CloseHandle(job);
            return Err(format!("Không quản lý được tiến trình Zalo (mã lỗi {err})"));
        }
        ResumeThread(pi.hThread);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        log(&format!("Đã mở Zalo pid={} profile={}", pi.dwProcessId, profile.display()));
        Ok(Session {
            job,
            pid: pi.dwProcessId,
            hwnd: null_mut(),
            mode: None,
            orig_style: 0,
            orig_exstyle: 0,
            state: State::Starting,
            started: Instant::now(),
            hidden_ticks: 0,
            unread: 0,
            ever_window: false,
        })
    }
}

pub fn stop(s: &Session) {
    unsafe {
        TerminateJobObject(s.job, 0);
        // chờ tối đa 3 giây cho các tiến trình thoát hẳn (giải phóng file khóa)
        for _ in 0..30 {
            if job_pids(s.job).is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        CloseHandle(s.job);
    }
}

// ---------------- tìm cửa sổ ----------------

#[derive(Clone, Debug)]
pub struct WinInfo {
    pub hwnd: HWND,
    pub area: i64,
    pub visible: bool,
    pub class: String,
    pub title: String,
}

struct EnumCtx {
    pids: Vec<u32>,
    out: Vec<WinInfo>,
}

unsafe extern "system" fn enum_cb(hwnd: HWND, lp: LPARAM) -> BOOL {
    let ctx = &mut *(lp as *mut EnumCtx);
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if !ctx.pids.contains(&pid) {
        return 1;
    }
    if !GetWindow(hwnd, GW_OWNER).is_null() {
        return 1;
    }
    let mut r: RECT = std::mem::zeroed();
    GetWindowRect(hwnd, &mut r);
    let (wd, ht) = ((r.right - r.left) as i64, (r.bottom - r.top) as i64);
    let mut cls = [0u16; 128];
    GetClassNameW(hwnd, cls.as_mut_ptr(), 128);
    let mut title = [0u16; 256];
    GetWindowTextW(hwnd, title.as_mut_ptr(), 256);
    let class = from_w(&cls);
    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    if ex & WS_EX_TOOLWINDOW != 0 {
        return 1;
    }
    if class.contains("tooltip") || class.starts_with("IME") || class == "MSCTFIME UI" {
        return 1;
    }
    if wd < 250 || ht < 250 {
        return 1;
    }
    ctx.out.push(WinInfo { hwnd, area: wd * ht, visible: IsWindowVisible(hwnd) != 0, class, title: from_w(&title) });
    1
}

pub fn list_windows(pids: &[u32]) -> Vec<WinInfo> {
    if pids.is_empty() {
        return vec![];
    }
    let mut ctx = EnumCtx { pids: pids.to_vec(), out: vec![] };
    unsafe {
        EnumWindows(Some(enum_cb), &mut ctx as *mut _ as LPARAM);
    }
    ctx.out
}

/// Cửa sổ chính tốt nhất: ưu tiên đang hiện, lớn nhất.
pub fn best_window(pids: &[u32], include_hidden: bool, exclude: HWND) -> Option<WinInfo> {
    let mut v: Vec<WinInfo> = list_windows(pids)
        .into_iter()
        .filter(|x| x.hwnd != exclude)
        .filter(|x| x.visible || (include_hidden && !x.title.is_empty()))
        .collect();
    v.sort_by_key(|x| {
        (std::cmp::Reverse(x.visible), std::cmp::Reverse(x.class == "Chrome_WidgetWin_1"), std::cmp::Reverse(x.area))
    });
    v.into_iter().next()
}

/// "(3) Zalo" -> 3
pub fn parse_unread(title: &str) -> u32 {
    let t = title.trim();
    if let Some(rest) = t.strip_prefix('(') {
        if let Some(end) = rest.find(')') {
            let n = rest[..end].trim_end_matches('+');
            return n.parse().unwrap_or(0);
        }
    }
    0
}

pub fn window_title(hwnd: HWND) -> String {
    let mut title = [0u16; 256];
    unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), 256) };
    from_w(&title)
}

// ---------------- tìm Zalo.exe ----------------

fn reg_string(root: HKEY, sub: &str, val: &str) -> Option<String> {
    let mut buf = vec![0u16; 1024];
    let mut len = (buf.len() * 2) as u32;
    let sub_w = w(sub);
    let val_w = w(val);
    let r = unsafe {
        RegGetValueW(
            root,
            sub_w.as_ptr(),
            val_w.as_ptr(),
            RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
            null_mut(),
            buf.as_mut_ptr() as *mut _,
            &mut len,
        )
    };
    if r == 0 {
        Some(from_w(&buf))
    } else {
        None
    }
}

fn reg_uninstall_candidates(out: &mut Vec<PathBuf>) {
    let roots = [
        (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];
    for (root, path) in roots {
        unsafe {
            let mut key: HKEY = null_mut();
            let pw = w(path);
            if RegOpenKeyExW(root, pw.as_ptr(), 0, KEY_READ, &mut key) != 0 {
                continue;
            }
            let mut i = 0u32;
            loop {
                let mut name = [0u16; 256];
                let mut nlen = 256u32;
                let r = RegEnumKeyExW(key, i, name.as_mut_ptr(), &mut nlen, null(), null_mut(), null_mut(), null_mut());
                if r != 0 {
                    break;
                }
                i += 1;
                let sub = format!("{}\\{}", path, from_w(&name));
                let disp = reg_string(root, &sub, "DisplayName").unwrap_or_default();
                if !disp.to_lowercase().starts_with("zalo") {
                    continue;
                }
                if let Some(loc) = reg_string(root, &sub, "InstallLocation") {
                    out.push(PathBuf::from(loc.trim_matches('"')).join("Zalo.exe"));
                }
                if let Some(icon) = reg_string(root, &sub, "DisplayIcon") {
                    let icon = icon.split(',').next().unwrap_or("").trim_matches('"').to_string();
                    if icon.to_lowercase().ends_with(".exe") {
                        out.push(PathBuf::from(icon));
                    }
                }
            }
            RegCloseKey(key);
        }
    }
}

pub fn find_zalo(user_choice: Option<&str>) -> Option<PathBuf> {
    if let Some(u) = user_choice {
        let p = PathBuf::from(u);
        if p.is_file() {
            return Some(p);
        }
    }
    let mut c: Vec<PathBuf> = vec![];
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    if let Some(l) = env("LOCALAPPDATA") {
        c.push(l.join("Programs\\Zalo\\Zalo.exe"));
        c.push(l.join("Zalo\\Zalo.exe"));
    }
    for k in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(p) = env(k) {
            c.push(p.join("Zalo\\Zalo.exe"));
        }
    }
    reg_uninstall_candidates(&mut c);
    for p in &c {
        if p.is_file() && p.file_name().map(|n| n.to_string_lossy().to_lowercase().contains("zalo")).unwrap_or(false) {
            return Some(p.clone());
        }
    }
    None
}
