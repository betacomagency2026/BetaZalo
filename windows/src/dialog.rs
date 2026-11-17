use std::cell::RefCell;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::config::COLORS;
use crate::gfx::*;
use crate::util::{from_w, initials, w};

const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;
const ID_EDIT: i32 = 100;

struct Dlg {
    color: usize,
    done: bool,
    result: Option<(String, usize)>,
    edit: HWND,
    swatches: Vec<RECT>,
    preview: RECT,
    scale: f32,
    font: HFONT,
    font_b: HFONT,
    font_av: HFONT,
    heading: String,
}

thread_local! {
    static DLG: RefCell<Option<Dlg>> = const { RefCell::new(None) };
}

fn s(scale: f32, v: i32) -> i32 {
    (v as f32 * scale).round() as i32
}

unsafe extern "system" fn dlg_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut cr: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut cr);
            let mut cv = Canvas::new(hdc);
            cv.fill_rect(cr, 0xFFFFFF);
            let _ = DLG.try_with(|d| {
                if let Ok(d) = d.try_borrow() {
                    if let Some(d) = d.as_ref() {
                        let sc = d.scale;
                        cv.text(
                            rc(s(sc, 24), s(sc, 18), cr.right - s(sc, 24), s(sc, 44)),
                            &d.heading,
                            d.font_b,
                            0x111827,
                            DT_SINGLELINE | DT_VCENTER,
                        );
                        cv.text(
                            rc(s(sc, 24), s(sc, 56), cr.right, s(sc, 76)),
                            "Tên tài khoản",
                            d.font,
                            0x4B5563,
                            DT_SINGLELINE | DT_VCENTER,
                        );
                        cv.text(
                            rc(s(sc, 24), s(sc, 124), cr.right, s(sc, 144)),
                            "Màu nhận diện",
                            d.font,
                            0x4B5563,
                            DT_SINGLELINE | DT_VCENTER,
                        );
                        for (i, r) in d.swatches.iter().enumerate() {
                            let dd = r.right - r.left;
                            if i == d.color {
                                cv.fill_circle(r.left - s(sc, 3), r.top - s(sc, 3), dd + s(sc, 6), 0x4F46E5);
                                cv.fill_circle(r.left - s(sc, 1), r.top - s(sc, 1), dd + s(sc, 2), 0xFFFFFF);
                            }
                            cv.fill_circle(r.left, r.top, dd, COLORS[i].0);
                            cv.stroke_round(r.left, r.top, dd, dd, dd / 2, COLORS[i].1, 1.0);
                        }
                        let mut buf = [0u16; 128];
                        GetWindowTextW(d.edit, buf.as_mut_ptr(), 128);
                        let name = from_w(&buf);
                        let p = d.preview;
                        let dd = p.right - p.left;
                        cv.fill_circle(p.left, p.top, dd, COLORS[d.color].0);
                        cv.text(
                            p,
                            &initials(if name.trim().is_empty() { "?" } else { &name }),
                            d.font_av,
                            COLORS[d.color].1,
                            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
                        );
                    }
                }
            });
            cv.finish();
            EndPaint(hwnd, &ps);
            0
        }
        WM_LBUTTONUP => {
            let x = (lp & 0xFFFF) as i16 as i32;
            let y = ((lp >> 16) & 0xFFFF) as i16 as i32;
            DLG.with(|d| {
                if let Some(d) = d.borrow_mut().as_mut() {
                    for (i, r) in d.swatches.iter().enumerate() {
                        let mut r2 = *r;
                        r2.left -= 4;
                        r2.top -= 4;
                        r2.right += 4;
                        r2.bottom += 4;
                        if contains(&r2, x, y) {
                            d.color = i;
                        }
                    }
                }
            });
            InvalidateRect(hwnd, null(), 0);
            0
        }
        WM_SETCURSOR => {
            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);
            ScreenToClient(hwnd, &mut pt);
            let over = DLG.with(|d| {
                d.borrow().as_ref().map(|d| d.swatches.iter().any(|r| contains(r, pt.x, pt.y))).unwrap_or(false)
            });
            if over && (lp & 0xFFFF) as u32 == HTCLIENT {
                SetCursor(LoadCursorW(null_mut(), IDC_HAND));
                return 1;
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_CTLCOLORSTATIC => {
            SetBkColor(wp as HDC, cref(0xFFFFFF));
            GetStockObject(WHITE_BRUSH) as LRESULT
        }
        WM_COMMAND => {
            let id = (wp & 0xFFFF) as i32;
            let code = ((wp >> 16) & 0xFFFF) as u32;
            if id == ID_OK {
                do_ok(hwnd);
            } else if id == ID_CANCEL {
                DLG.with(|d| {
                    if let Some(d) = d.borrow_mut().as_mut() {
                        d.done = true;
                    }
                });
            } else if id == ID_EDIT && code == EN_CHANGE {
                InvalidateRect(hwnd, null(), 0);
            }
            0
        }
        WM_CLOSE => {
            DLG.with(|d| {
                if let Some(d) = d.borrow_mut().as_mut() {
                    d.done = true;
                }
            });
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn do_ok(hwnd: HWND) {
    DLG.with(|d| {
        if let Some(d) = d.borrow_mut().as_mut() {
            let mut buf = [0u16; 128];
            GetWindowTextW(d.edit, buf.as_mut_ptr(), 128);
            let name = from_w(&buf).trim().to_string();
            if name.is_empty() {
                MessageBeep(MB_ICONWARNING);
                SetFocus(d.edit);
                return;
            }
            d.result = Some((name, d.color));
            d.done = true;
        }
    });
    let _ = hwnd;
}

pub fn account_dialog(owner: HWND, heading: &str, name: &str, color: usize, ok_label: &str) -> Option<(String, usize)> {
    unsafe {
        let hinst = GetModuleHandleW(null());
        let cls = w("BetacomDialog");
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(dlg_proc);
        wc.hInstance = hinst;
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc.hbrBackground = GetStockObject(WHITE_BRUSH) as HBRUSH;
        wc.lpszClassName = cls.as_ptr();
        RegisterClassExW(&wc);

        let dpi = GetDpiForWindow(owner).max(96);
        let sc = dpi as f32 / 96.0;
        let (cw, ch) = (s(sc, 420), s(sc, 252));
        let style = WS_POPUP | WS_CAPTION | WS_SYSMENU;
        let mut r = rc(0, 0, cw, ch);
        windows_sys::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, style, 0, WS_EX_DLGMODALFRAME, dpi);
        let (ww, wh) = (r.right - r.left, r.bottom - r.top);
        let mut or: RECT = std::mem::zeroed();
        GetWindowRect(owner, &mut or);
        let x = or.left + ((or.right - or.left) - ww) / 2;
        let y = or.top + ((or.bottom - or.top) - wh) / 3;
        let title = w(heading);
        let dlg = CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            cls.as_ptr(),
            title.as_ptr(),
            style,
            x,
            y,
            ww,
            wh,
            owner,
            null_mut(),
            hinst,
            null(),
        );

        let font = make_font(s(sc, 14), 400, "Segoe UI");
        let font_b = make_font(s(sc, 18), 600, "Segoe UI");
        let font_av = make_font(s(sc, 14), 600, "Segoe UI");
        let mk = |class: &str, text: &str, st: u32, ex: u32, x: i32, y: i32, wd: i32, ht: i32, id: i32| -> HWND {
            let c = w(class);
            let t = w(text);
            let h = CreateWindowExW(
                ex,
                c.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE | st,
                x,
                y,
                wd,
                ht,
                dlg,
                id as isize as HMENU,
                hinst,
                null(),
            );
            SendMessageW(h, WM_SETFONT, font as WPARAM, 1);
            h
        };
        let pad = s(sc, 24);
        let av = s(sc, 36);
        let edit = mk(
            "EDIT",
            name,
            WS_TABSTOP | ES_AUTOHSCROLL as u32,
            WS_EX_CLIENTEDGE,
            pad,
            s(sc, 80),
            cw - pad * 2 - av - s(sc, 12),
            s(sc, 30),
            ID_EDIT,
        );
        let cue = w("Ví dụ: Bán hàng 1");
        SendMessageW(edit, EM_SETCUEBANNER, 1, cue.as_ptr() as LPARAM);
        SendMessageW(edit, EM_LIMITTEXT, 40, 0);
        let preview = rc(cw - pad - av, s(sc, 77), cw - pad, s(sc, 77) + av);

        let mut swatches = vec![];
        let d = s(sc, 26);
        for i in 0..COLORS.len() as i32 {
            let x = pad + s(sc, 4) + i * (d + s(sc, 14));
            swatches.push(rc(x, s(sc, 154), x + d, s(sc, 154) + d));
        }
        let bw = s(sc, 96);
        let bh = s(sc, 32);
        let by = ch - bh - s(sc, 18);
        mk("BUTTON", "Hủy", WS_TABSTOP | BS_PUSHBUTTON as u32, 0, cw - pad - bw * 2 - s(sc, 10), by, bw, bh, ID_CANCEL);
        mk("BUTTON", ok_label, WS_TABSTOP | BS_DEFPUSHBUTTON as u32, 0, cw - pad - bw, by, bw, bh, ID_OK);

        DLG.with(|dd| {
            *dd.borrow_mut() = Some(Dlg {
                color: color % COLORS.len(),
                done: false,
                result: None,
                edit,
                swatches,
                preview,
                scale: sc,
                font,
                font_b,
                font_av,
                heading: heading.to_string(),
            })
        });

        EnableWindow(owner, 0);
        ShowWindow(dlg, SW_SHOW);
        SetFocus(edit);
        SendMessageW(edit, EM_SETSEL, 0, -1);

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let done = DLG.with(|d| d.borrow().as_ref().map(|d| d.done).unwrap_or(true));
            if done {
                break;
            }
            if GetMessageW(&mut msg, null_mut(), 0, 0) <= 0 {
                PostQuitMessage(0);
                break;
            }
            let cancel_focused = GetDlgCtrlID(GetFocus()) == ID_CANCEL;
            if msg.message == WM_KEYDOWN && msg.wParam == VK_RETURN as usize && !cancel_focused {
                do_ok(dlg);
                continue;
            }
            if msg.message == WM_KEYDOWN
                && (msg.wParam == VK_ESCAPE as usize || (msg.wParam == VK_RETURN as usize && cancel_focused))
            {
                DLG.with(|d| {
                    if let Some(d) = d.borrow_mut().as_mut() {
                        d.done = true;
                    }
                });
                continue;
            }
            if IsDialogMessageW(dlg, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        EnableWindow(owner, 1);
        SetForegroundWindow(owner);
        DestroyWindow(dlg);
        let res = DLG.with(|d| d.borrow_mut().take()).and_then(|d| {
            DeleteObject(d.font as _);
            DeleteObject(d.font_b as _);
            DeleteObject(d.font_av as _);
            d.result
        });
        res
    }
}
