//! Biểu tượng khay hệ thống + thông báo góc phải màn hình.

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub const WM_TRAY: u32 = 0x8000 + 10;
const TRAY_ID: u32 = 1;

fn copy(dst: &mut [u16], s: &str) {
    let v: Vec<u16> = s.encode_utf16().collect();
    let n = v.len().min(dst.len() - 1);
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
        let hinst = GetModuleHandleW(std::ptr::null());
        d.hIcon = LoadImageW(
            hinst,
            crate::util::int_resource(1),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            0,
        ) as _;
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
        let hinst = GetModuleHandleW(std::ptr::null());
        d.hBalloonIcon = LoadImageW(
            hinst,
            crate::util::int_resource(1),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
            0,
        ) as _;
        Shell_NotifyIconW(NIM_MODIFY, &d);
    }
}
