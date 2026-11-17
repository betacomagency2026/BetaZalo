//! Biểu tượng khay hệ thống + thông báo góc phải màn hình.

use std::cell::Cell;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub const WM_TRAY: u32 = 0x8000 + 10;
const TRAY_ID: u32 = 1;

thread_local! {
    static SMALL_ICON: Cell<HICON> = const { Cell::new(null_mut()) };
    static LARGE_ICON: Cell<HICON> = const { Cell::new(null_mut()) };
}

fn cached_icon(
    slot: &'static std::thread::LocalKey<Cell<HICON>>,
    cx: SYSTEM_METRICS_INDEX,
    cy: SYSTEM_METRICS_INDEX,
) -> HICON {
    slot.with(|c| {
        if c.get().is_null() {
            let icon = unsafe {
                LoadImageW(
                    GetModuleHandleW(null()),
                    crate::util::int_resource(1),
                    IMAGE_ICON,
                    GetSystemMetrics(cx),
                    GetSystemMetrics(cy),
                    0,
                )
            };
            c.set(icon as HICON);
        }
        c.get()
    })
}

fn copy(dst: &mut [u16], s: &str) {
    let v: Vec<u16> = s.encode_utf16().collect();
    let mut n = v.len().min(dst.len() - 1);
    if n < v.len() && n > 0 && (0xD800..0xDC00).contains(&v[n - 1]) {
        n -= 1;
    }
    dst[..n].copy_from_slice(&v[..n]);
    dst[n] = 0;
}

fn base(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut d: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    d.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    d.hWnd = hwnd;
    d.uID = TRAY_ID;
    d
}

pub fn add(hwnd: HWND, tip: &str) {
    unsafe {
        let mut d = base(hwnd);
        d.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE | NIF_SHOWTIP;
        d.uCallbackMessage = WM_TRAY;
        d.hIcon = cached_icon(&SMALL_ICON, SM_CXSMICON, SM_CYSMICON);
        copy(&mut d.szTip, tip);
        Shell_NotifyIconW(NIM_ADD, &d);
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &d);
    }
}

pub fn set_tip(hwnd: HWND, tip: &str) {
    unsafe {
        let mut d = base(hwnd);
        d.uFlags = NIF_TIP | NIF_SHOWTIP;
        copy(&mut d.szTip, tip);
        Shell_NotifyIconW(NIM_MODIFY, &d);
    }
}

pub fn remove(hwnd: HWND) {
    unsafe {
        let d = base(hwnd);
        Shell_NotifyIconW(NIM_DELETE, &d);
    }
}

/// Hiện thông báo ở góc phải màn hình (Windows 10/11 hiển thị dạng toast).
pub fn notify(hwnd: HWND, title: &str, text: &str) {
    unsafe {
        let mut d = base(hwnd);
        d.uFlags = NIF_INFO;
        copy(&mut d.szInfoTitle, title);
        copy(&mut d.szInfo, if text.is_empty() { " " } else { text });
        d.dwInfoFlags = NIIF_USER | NIIF_LARGE_ICON | NIIF_RESPECT_QUIET_TIME;
        d.hBalloonIcon = cached_icon(&LARGE_ICON, SM_CXICON, SM_CYICON);
        Shell_NotifyIconW(NIM_MODIFY, &d);
    }
}
