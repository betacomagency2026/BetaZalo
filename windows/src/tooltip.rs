//! Chú thích nhỏ khi rê chuột (dùng cho thanh tài khoản thu gọn).

use super::*;
use windows_sys::Win32::UI::Controls::*;

pub(crate) fn create_tip(owner: HWND) -> HWND {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES,
        };
        InitCommonControlsEx(&icc);
        let cls = w("tooltips_class32");
        let tip = CreateWindowExW(
            WS_EX_TOPMOST,
            cls.as_ptr(),
            null(),
            WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP,
            0,
            0,
            0,
            0,
            owner,
            null_mut(),
            GetModuleHandleW(null()),
            null(),
        );
        if tip.is_null() {
            return tip;
        }
        let mut empty = w("");
        let mut ti: TTTOOLINFOW = std::mem::zeroed();
        ti.cbSize = std::mem::size_of::<TTTOOLINFOW>() as u32;
        ti.uFlags = TTF_TRACK | TTF_ABSOLUTE;
        ti.hwnd = owner;
        ti.uId = 1;
        ti.lpszText = empty.as_mut_ptr();
        SendMessageW(tip, TTM_ADDTOOLW, 0, &ti as *const _ as LPARAM);
        SendMessageW(tip, TTM_SETMAXTIPWIDTH, 0, 400);
        tip
    }
}

pub(crate) fn tool_info(tip: HWND, text: &mut Vec<u16>) -> windows_sys::Win32::UI::Controls::TTTOOLINFOW {
    let mut ti: TTTOOLINFOW = unsafe { std::mem::zeroed() };
    ti.cbSize = std::mem::size_of::<TTTOOLINFOW>() as u32;
    ti.hwnd = unsafe { GetWindow(tip, GW_OWNER) };
    ti.uId = 1;
    ti.lpszText = text.as_mut_ptr();
    ti
}

pub(crate) fn show_tip(tip: HWND, text: &str, x: i32, y: i32) {
    if tip.is_null() {
        return;
    }
    unsafe {
        let mut t = w(text);
        let ti = tool_info(tip, &mut t);
        SendMessageW(tip, TTM_UPDATETIPTEXTW, 0, &ti as *const _ as LPARAM);
        SendMessageW(tip, TTM_TRACKPOSITION, 0, ((y as u32 as usize) << 16 | (x as u16 as usize)) as LPARAM);
        SendMessageW(tip, TTM_TRACKACTIVATE, 1, &ti as *const _ as LPARAM);
    }
}

pub(crate) fn hide_tip(tip: HWND) {
    if tip.is_null() {
        return;
    }
    unsafe {
        let mut t = w("");
        let ti = tool_info(tip, &mut t);
        SendMessageW(tip, TTM_TRACKACTIVATE, 0, &ti as *const _ as LPARAM);
    }
}
