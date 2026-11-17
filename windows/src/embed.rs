//! Gắn cửa sổ Zalo vào MultiZ: nhúng (cửa sổ con) hoặc gắn kèm (cửa sổ nổi đi theo).

use super::*;

pub(crate) fn apply_mode(s: &mut Session, embed: bool, pane: HWND, main: HWND) {
    unsafe {
        let h = s.hwnd;
        if IsIconic(h) != 0 || IsZoomed(h) != 0 {
            ShowWindow(h, SW_RESTORE);
        }
        if embed {
            let st = s.orig_style as u32;
            let new_st = (st & !(WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU))
                | WS_CHILD
                | WS_CLIPSIBLINGS;
            SetWindowLongPtrW(h, GWL_STYLE, new_st as isize);
            let ex = s.orig_exstyle as u32 & !(WS_EX_APPWINDOW | WS_EX_WINDOWEDGE | WS_EX_DLGMODALFRAME);
            SetWindowLongPtrW(h, GWL_EXSTYLE, ex as isize);
            let prev = SetParent(h, pane);
            if prev.is_null() {
                log(&format!("SetParent lỗi {} — hãy thử chế độ cửa sổ gắn kèm", GetLastError()));
            }
            SetWindowPos(h, HWND_TOP, 0, 0, 10, 10, SWP_FRAMECHANGED | SWP_NOACTIVATE);
            log(&format!("Nhúng xong: parent khớp = {}", GetParent(h) == pane));
        } else {
            let ex = s.orig_exstyle as u32 & !WS_EX_APPWINDOW;
            SetWindowLongPtrW(h, GWL_EXSTYLE, ex as isize);
            SetWindowLongPtrW(h, GWLP_HWNDPARENT, main as isize);
        }
        s.mode = Some(embed);
    }
}

pub(crate) fn release(s: &mut Session) {
    unsafe {
        let h = s.hwnd;
        if h.is_null() || IsWindow(h) == 0 {
            s.mode = None;
            return;
        }
        match s.mode {
            Some(true) => {
                SetParent(h, null_mut());
                SetWindowLongPtrW(h, GWL_STYLE, s.orig_style);
                SetWindowLongPtrW(h, GWL_EXSTYLE, s.orig_exstyle);
                SetWindowPos(
                    h,
                    null_mut(),
                    80,
                    80,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            Some(false) => {
                SetWindowLongPtrW(h, GWLP_HWNDPARENT, 0);
                SetWindowLongPtrW(h, GWL_EXSTYLE, s.orig_exstyle);
            }
            None => {}
        }
        s.mode = None;
    }
}

/// Hiện cửa sổ Zalo của tài khoản đang chọn, phủ kín khung bên phải.
pub(crate) fn place(s: &mut Session, embed: bool, pane: HWND, main: HWND) {
    unsafe {
        let h = s.hwnd;
        if h.is_null() || IsHungAppWindow(h) != 0 {
            return;
        }
        let mut pr: RECT = std::mem::zeroed();
        GetClientRect(pane, &mut pr);
        if embed {
            SetWindowPos(h, HWND_TOP, 0, 0, pr.right, pr.bottom, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        } else {
            if IsIconic(main) != 0 || IsWindowVisible(main) == 0 {
                return;
            }
            let mut wr: RECT = std::mem::zeroed();
            GetWindowRect(pane, &mut wr);
            SetWindowPos(
                h,
                null_mut(),
                wr.left,
                wr.top,
                wr.right - wr.left,
                wr.bottom - wr.top,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_SHOWWINDOW,
            );
        }
    }
}

/// Cất cửa sổ Zalo của tài khoản không chọn.
/// Chế độ nhúng: vẫn "hiển thị" nhưng nằm ngoài khung nhìn để Zalo tiếp tục cập nhật.
pub(crate) fn park(s: &mut Session, embed: bool, pane: HWND) {
    unsafe {
        let h = s.hwnd;
        if h.is_null() || IsHungAppWindow(h) != 0 {
            return;
        }
        if embed {
            let mut pr: RECT = std::mem::zeroed();
            GetClientRect(pane, &mut pr);
            let wd = pr.right.max(400);
            SetWindowPos(h, HWND_BOTTOM, -wd - 200, 0, wd, pr.bottom.max(300), SWP_NOACTIVATE | SWP_SHOWWINDOW);
        } else {
            ShowWindow(h, SW_HIDE);
        }
    }
}

/// Cửa sổ Zalo bị nhúng làm cửa sổ con sẽ không bao giờ nhận được tín hiệu "được kích hoạt"
/// từ Windows, nên Zalo (Chromium) không nhận bàn phím. Ta tự gửi tín hiệu kích hoạt + focus.
pub(crate) fn activate_child(h: HWND) {
    unsafe {
        if h.is_null() || IsWindow(h) == 0 || IsHungAppWindow(h) != 0 {
            return;
        }
        PostMessageW(h, WM_NCACTIVATE, 1, 0);
        PostMessageW(h, WM_ACTIVATE, WA_ACTIVE as usize, 0);
        SetFocus(h);
    }
}

pub(crate) fn deactivate_child(h: HWND) {
    unsafe {
        if h.is_null() || IsWindow(h) == 0 {
            return;
        }
        PostMessageW(h, WM_ACTIVATE, WA_INACTIVE as usize, 0);
        PostMessageW(h, WM_NCACTIVATE, 0, 0);
    }
}

pub(crate) fn adopt(s: &mut Session, wi: &session::WinInfo, embed: bool, pane: HWND, main: HWND, is_sel: bool) {
    unsafe {
        s.hwnd = wi.hwnd;
        s.orig_style = GetWindowLongPtrW(wi.hwnd, GWL_STYLE);
        s.orig_exstyle = GetWindowLongPtrW(wi.hwnd, GWL_EXSTYLE);
        s.state = State::Running;
        s.ever_window = true;
        s.hidden_ticks = 0;
        log(&format!(
            "Gắn cửa sổ pid={} class='{}' diện tích={} ({})",
            s.pid,
            wi.class,
            wi.area,
            if embed { "nhúng" } else { "gắn kèm" }
        ));
        apply_mode(s, embed, pane, main);
        if is_sel {
            place(s, embed, pane, main);
            PostMessageW(main, WM_APP_FOCUS, 0, 0);
        } else {
            park(s, embed, pane);
        }
    }
}
