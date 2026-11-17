use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

pub fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn int_resource(id: u16) -> *const u16 {
    id as usize as *const u16
}

pub fn from_w(buf: &[u16]) -> String {
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..n])
}

static ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn data_root() -> &'static PathBuf {
    ROOT.get_or_init(|| {
        let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let p = base.join("Betacom");
        if !p.exists() {
            for old_name in ["MultiZ", "MultiZalo", "Zaloo"] {
                let old = base.join(old_name);
                if old.is_dir() {
                    if std::fs::rename(&old, &p).is_err() {
                        return old;
                    }
                    break;
                }
            }
        }
        let _ = std::fs::create_dir_all(&p);
        p
    })
}

pub fn is_safe_id(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn profiles_dir() -> PathBuf {
    data_root().join("profiles")
}

pub fn log_path() -> PathBuf {
    data_root().join("log.txt")
}

fn local_time() -> SYSTEMTIME {
    unsafe {
        let mut t = std::mem::zeroed();
        GetLocalTime(&mut t);
        t
    }
}

pub fn stamp() -> String {
    let t = local_time();
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
}

static LOG_LOCK: Mutex<()> = Mutex::new(());

pub fn log(msg: &str) {
    let t = local_time();
    let line = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}  {}\r\n",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, msg
    );
    let _guard = LOG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let p = log_path();
    if let Ok(m) = std::fs::metadata(&p) {
        if m.len() > 1_000_000 {
            let _ = std::fs::rename(&p, data_root().join("log.old.txt"));
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let mut s = String::new();
    match words.len() {
        0 => s.push('?'),
        1 => {
            for c in words[0].chars().take(2) {
                s.extend(c.to_uppercase());
            }
        }
        _ => {
            for wd in [words[0], words[words.len() - 1]] {
                if let Some(c) = wd.chars().next() {
                    s.extend(c.to_uppercase());
                }
            }
        }
    }
    s
}

pub fn new_id() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("acc{:x}", t)
}
