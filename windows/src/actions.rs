//! Các thao tác có hộp thoại / menu: thêm, sửa, xóa tài khoản, cài đặt, khay hệ thống.

use super::*;

pub(crate) fn hwnd_main() -> HWND {
    with_app(|a| a.hwnd).unwrap_or(null_mut())
}

pub(crate) fn msgbox(text: &str, flags: u32) -> i32 {
    let t = w(text);
    let c = w(APP_NAME);
    unsafe { MessageBoxW(hwnd_main(), t.as_ptr(), c.as_ptr(), flags) }
}

pub(crate) fn open_path(p: &str) {
    let op = w("open");
    let f = w(p);
    unsafe { ShellExecuteW(hwnd_main(), op.as_ptr(), f.as_ptr(), null(), null(), SW_SHOWNORMAL) };
}

pub(crate) fn add_account() {
    let main = hwnd_main();
    let next_color = with_app(|a| a.cfg.accounts.len() % COLORS.len()).unwrap_or(0);
    let n = with_app(|a| a.cfg.accounts.len()).unwrap_or(0);
    let suggestion = format!("Zalo {}", n + 1);
    if let Some((name, color)) = dialog::account_dialog(main, "Thêm tài khoản Zalo", &suggestion, next_color, "Thêm")
    {
        with_app(|a| {
            let acc = Account { id: util::new_id(), name, color, autostart: true, muted: false };
            log(&format!("Thêm tài khoản {} ({})", acc.name, acc.id));
            a.cfg.accounts.push(acc);
            let idx = a.cfg.accounts.len() - 1;
            a.scroll = idx.saturating_sub(2);
            let id = a.cfg.accounts[idx].id.clone();
            a.select(idx);
            a.start(&id);
        });
    }
}

pub(crate) fn edit_account(idx: usize) {
    let main = hwnd_main();
    let Some(acc) = with_app(|a| a.cfg.accounts.get(idx).cloned()).flatten() else { return };
    if let Some((name, color)) = dialog::account_dialog(main, "Đổi tên và màu", &acc.name, acc.color, "Lưu") {
        with_app(|a| {
            if let Some(x) = a.cfg.accounts.get_mut(idx) {
                x.name = name;
                x.color = color;
            }
            a.cfg.save();
            a.invalidate();
        });
    }
}

pub(crate) fn change_avatar(id: &str) {
    let Some(file) = avatar::pick_file(hwnd_main()) else { return };
    match avatar::import(&file, id) {
        Ok(()) => {
            with_app(|a| {
                match avatar::load(id) {
                    Some(img) => {
                        a.avatars.insert(id.to_string(), img);
                    }
                    None => {
                        a.avatars.remove(id);
                    }
                }
                a.invalidate();
            });
        }
        Err(e) => {
            log(&format!("Đổi ảnh đại diện lỗi: {e}"));
            msgbox(&e, MB_OK | MB_ICONWARNING);
        }
    }
}

pub(crate) fn delete_account(idx: usize) {
    let Some(acc) = with_app(|a| a.cfg.accounts.get(idx).cloned()).flatten() else { return };
    let q = format!(
        "Xóa tài khoản \"{}\" khỏi {APP_NAME}?\n\nDữ liệu đăng nhập và tin nhắn lưu trên máy của tài khoản này sẽ bị xóa. Tài khoản Zalo của bạn không bị ảnh hưởng: bạn có thể thêm lại và đăng nhập bằng mã QR bất cứ lúc nào.",
        acc.name
    );
    if msgbox(&q, MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2) != IDYES {
        return;
    }
    let (shared, job) = with_app(|a| {
        a.stop(&acc.id);
        a.avatars.remove(&acc.id);
        let shared = a.cfg.accounts.iter().filter(|x| x.id == acc.id).count() > 1;
        (shared, a.stopping.remove(&acc.id).map(|s| s.job))
    })
    .unwrap_or((true, None));
    if let Some(job) = job {
        if !session::wait_exit(job, std::time::Duration::from_secs(3)) {
            log(&format!("Phiên {} chưa thoát hẳn trước khi xóa dữ liệu", acc.id));
        }
        unsafe { CloseHandle(job) };
    }
    let dir = util::profiles_dir().join(&acc.id);
    let mut leftover = None;
    if !util::is_safe_id(&acc.id) || shared {
        log(&format!("Không xóa thư mục dữ liệu vì id không hợp lệ hoặc bị trùng: {:?}", acc.id));
    } else if dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            log(&format!("Xóa {} lỗi, thử lại: {e}", dir.display()));
            std::thread::sleep(std::time::Duration::from_secs(1));
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                log(&format!("Không xóa hết được {}: {e}", dir.display()));
                leftover = Some(dir);
            }
        }
    }
    with_app(|a| {
        let pos = match a.cfg.accounts.get(idx) {
            Some(x) if x.id == acc.id => Some(idx),
            _ => a.cfg.index_of(&acc.id),
        };
        if let Some(pos) = pos {
            a.cfg.accounts.remove(pos);
        }
        if a.cfg.selected.as_deref() == Some(&acc.id) && a.cfg.index_of(&acc.id).is_none() {
            a.cfg.selected = a.cfg.accounts.first().map(|x| x.id.clone());
        }
        a.cfg.save();
        log(&format!("Đã xóa tài khoản {} ({})", acc.name, acc.id));
        a.place_all();
        a.invalidate();
    });
    if let Some(dir) = leftover {
        msgbox(
            &format!(
                "Đã gỡ tài khoản khỏi danh sách nhưng chưa xóa hết dữ liệu. Hãy đóng các chương trình đang dùng thư mục này rồi xóa tay:\n\n{}",
                dir.display()
            ),
            MB_OK | MB_ICONWARNING,
        );
    }
}

pub(crate) fn pick_exe() {
    let main = hwnd_main();
    let mut file = [0u16; 1024];
    let filter: Vec<u16> = "Zalo.exe\0Zalo*.exe\0Ứng dụng (*.exe)\0*.exe\0\0".encode_utf16().collect();
    let title = w("Chọn file Zalo.exe");
    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = main;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file.as_mut_ptr();
    ofn.nMaxFile = file.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    if unsafe { GetOpenFileNameW(&mut ofn) } != 0 {
        let p = util::from_w(&file);
        with_app(|a| {
            a.cfg.zalo_path = Some(p.clone());
            a.cfg.save();
            a.zalo = session::find_zalo(Some(&p));
            log(&format!("Người dùng chọn Zalo.exe: {p}"));
            a.invalidate();
        });
    }
}

pub(crate) fn popup_menu(items: &[(u32, &str, bool, bool)], x: i32, y: i32) -> u32 {
    // items: (id, nhãn, checked, enabled); id 0 = vạch ngăn
    unsafe {
        let m = CreatePopupMenu();
        let mut keep: Vec<Vec<u16>> = vec![];
        for (id, label, checked, enabled) in items {
            if *id == 0 {
                AppendMenuW(m, MF_SEPARATOR, 0, null());
                continue;
            }
            let t = w(label);
            let mut fl = MF_STRING;
            if *checked {
                fl |= MF_CHECKED;
            }
            if !*enabled {
                fl |= MF_GRAYED;
            }
            AppendMenuW(m, fl, *id as usize, t.as_ptr());
            keep.push(t);
        }
        let main = hwnd_main();
        SetForegroundWindow(main);
        let r =
            TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_TOPALIGN, x, y, 0, main, null());
        PostMessageW(main, WM_NULL, 0, 0);
        DestroyMenu(m);
        r as u32
    }
}

// ---- khởi động cùng Windows ----
pub(crate) const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub(crate) fn migrate_autorun() {
    let sub = w(RUN_KEY);
    let mut had_old = false;
    for name in ["Zaloo", "MultiZ"] {
        let old = w(name);
        let mut len = 0u32;
        let found = unsafe {
            RegGetValueW(HKEY_CURRENT_USER, sub.as_ptr(), old.as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), &mut len)
                == 0
        };
        if found {
            unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, sub.as_ptr(), old.as_ptr()) };
            had_old = true;
        }
    }
    if had_old {
        set_autorun(true);
    } else if autorun_enabled() {
        // cập nhật đường dẫn nếu file .exe đã đổi tên / đổi chỗ
        set_autorun(true);
    }
}

pub(crate) fn autorun_enabled() -> bool {
    let sub = w(RUN_KEY);
    let val = w(APP_NAME);
    let mut len = 0u32;
    unsafe {
        RegGetValueW(HKEY_CURRENT_USER, sub.as_ptr(), val.as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), &mut len)
            == 0
    }
}

pub(crate) fn set_autorun(on: bool) {
    let sub = w(RUN_KEY);
    let val = w(APP_NAME);
    unsafe {
        if on {
            let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
            let data = w(&format!("\"{exe}\" --tray"));
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                sub.as_ptr(),
                val.as_ptr(),
                REG_SZ,
                data.as_ptr() as *const _,
                (data.len() * 2) as u32,
            );
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, sub.as_ptr(), val.as_ptr());
        }
    }
}

pub(crate) const LICENSE_TEXT: &str = "Giấy phép MIT (MIT License)

Copyright (c) 2026 Betacom
Copyright (c) 2026 Sudo Software — https://sudo.vn

Được phép sử dụng, sao chép, chỉnh sửa, gộp, xuất bản, phân phối, cấp phép lại và/hoặc bán bản sao của phần mềm này, miễn phí, với điều kiện giữ nguyên thông báo bản quyền và giấy phép này trong mọi bản sao.

PHẦN MỀM ĐƯỢC CUNG CẤP \"NGUYÊN TRẠNG\", KHÔNG KÈM BẤT KỲ BẢO ĐẢM NÀO. TÁC GIẢ KHÔNG CHỊU TRÁCH NHIỆM VỀ BẤT KỲ THIỆT HẠI NÀO PHÁT SINH TỪ VIỆC SỬ DỤNG PHẦN MỀM.

Lưu ý:
• Betacom là phần mềm độc lập, không phải sản phẩm chính thức và không liên kết với Zalo / VNG.
• \"Zalo\" là nhãn hiệu của chủ sở hữu tương ứng.
• Betacom chạy ứng dụng Zalo PC gốc do người dùng tự cài đặt; người dùng tự chịu trách nhiệm tuân thủ điều khoản sử dụng của Zalo.

Bản tiếng Anh đầy đủ của giấy phép có trong file LICENSE của mã nguồn.";

pub(crate) fn show_about() {
    let zalo = with_app(|a| a.zalo.as_ref().map(|p| p.display().to_string()))
        .flatten()
        .unwrap_or_else(|| "(chưa tìm thấy)".into());
    let text = format!(
        "{APP_NAME} {APP_VERSION}\n\
         Chạy nhiều tài khoản Zalo PC trên cùng một máy tính.\n\n\
         Phát triển bởi {DEVELOPER} — {DEVELOPER_URL}\n\
         Giấy phép MIT — miễn phí, mã nguồn mở, không quảng cáo.\n\n\
         Quyền riêng tư:\n\
         • {APP_NAME} không kết nối Internet và không thu thập dữ liệu gì.\n\
         • Mỗi tài khoản Zalo có một thư mục dữ liệu riêng trên máy bạn.\n\
         • {APP_NAME} không sửa và không chèn mã vào Zalo.\n\n\
         {APP_NAME} không phải sản phẩm chính thức của Zalo / VNG.\n\n\
         Zalo.exe: {zalo}\n\
         Dữ liệu: {}",
        util::data_root().display()
    );
    msgbox(&text, MB_OK | MB_ICONINFORMATION);
}

pub(crate) fn settings_menu(x: i32, y: i32) {
    let Some((embed, notify, hide, tray_close)) =
        with_app(|a| (a.cfg.embed, a.cfg.notify, a.cfg.hide_content, a.cfg.close_to_tray))
    else {
        return;
    };
    let autorun = autorun_enabled();
    let items = [
        (1, "Thông báo khi có tin nhắn mới", notify, true),
        (2, "Ẩn nội dung tin trong thông báo", hide, notify),
        (3, "Bấm X thì chạy ẩn dưới khay hệ thống", tray_close, true),
        (4, "Khởi động cùng Windows", autorun, true),
        (5, "Tắt thông báo trùng của Zalo gốc…", false, true),
        (0, "", false, true),
        (6, "Hiển thị: nhúng Zalo vào trong ứng dụng", embed, true),
        (7, "Hiển thị: cửa sổ Zalo gắn kèm (dùng khi nhúng bị lỗi)", !embed, true),
        (0, "", false, true),
        (8, "Chọn vị trí Zalo.exe…", false, true),
        (9, "Mở thư mục dữ liệu", false, true),
        (0, "", false, true),
        (16, "Kiểm tra phiên bản mới…", false, true),
        (12, "Trang chủ Betacom", false, true),
        (13, "Giấy phép", false, true),
        (14, "Giới thiệu Betacom", false, true),
    ];
    match popup_menu(&items, x, y) {
        1 => {
            with_app(|a| {
                a.cfg.notify = !a.cfg.notify;
                a.cfg.save();
            });
        }
        2 => {
            with_app(|a| {
                a.cfg.hide_content = !a.cfg.hide_content;
                a.cfg.save();
            });
        }
        3 => {
            with_app(|a| {
                a.cfg.close_to_tray = !a.cfg.close_to_tray;
                a.cfg.save();
            });
        }
        4 => set_autorun(!autorun),
        5 => {
            msgbox(
                &format!("Mỗi phiên Zalo vẫn tự hiện thông báo riêng, nhưng thông báo của Zalo không cho biết tin đến tài khoản nào.\n\nNếu thấy thông báo của {APP_NAME} hoạt động tốt, bạn có thể tắt thông báo của Zalo để không bị báo trùng:\nWindows Settings → System → Notifications → Zalo → Tắt.\n\nBấm OK để mở trang cài đặt thông báo của Windows."),
                MB_OK | MB_ICONINFORMATION,
            );
            open_path("ms-settings:notifications");
        }
        6 => {
            with_app(|a| a.set_mode(true));
        }
        7 => {
            with_app(|a| a.set_mode(false));
        }
        8 => pick_exe(),
        9 => open_path(&util::data_root().to_string_lossy()),
        16 => open_path(UPDATE_URL),
        12 => open_path(DEVELOPER_URL),
        13 => {
            msgbox(LICENSE_TEXT, MB_OK | MB_ICONINFORMATION);
        }
        14 => show_about(),
        _ => {}
    }
}

pub(crate) fn account_menu(idx: usize, x: i32, y: i32) {
    let Some((acc, running, has_avatar)) = with_app(|a| {
        a.cfg.accounts.get(idx).cloned().map(|acc| {
            let running = a.sessions.contains_key(&acc.id);
            let has_avatar = a.avatars.contains_key(&acc.id);
            (acc, running, has_avatar)
        })
    })
    .flatten() else {
        return;
    };
    let items = [
        (1, if running { "Khởi động lại" } else { "Mở Zalo" }, false, true),
        (2, "Tắt", false, running),
        (0, "", false, true),
        (3, "Đổi tên và màu…", false, true),
        (8, "Đổi ảnh đại diện…", false, true),
        (9, "Dùng lại chữ viết tắt", false, has_avatar),
        (7, "Tắt thông báo của tài khoản này", acc.muted, true),
        (4, "Tự mở khi khởi động Betacom", acc.autostart, true),
        (5, "Mở thư mục dữ liệu của tài khoản", false, true),
        (0, "", false, true),
        (6, "Xóa tài khoản…", false, true),
    ];
    match popup_menu(&items, x, y) {
        1 => {
            with_app(|a| {
                a.restart(&acc.id);
                a.select(idx);
            });
        }
        2 => {
            with_app(|a| a.stop(&acc.id));
        }
        3 => edit_account(idx),
        8 => change_avatar(&acc.id),
        9 => {
            avatar::remove(&acc.id);
            with_app(|a| {
                a.avatars.remove(&acc.id);
                a.invalidate();
            });
        }
        4 => {
            with_app(|a| {
                if let Some(x) = a.cfg.accounts.get_mut(idx) {
                    x.autostart = !x.autostart;
                }
                a.cfg.save();
            });
        }
        7 => {
            with_app(|a| {
                if let Some(x) = a.cfg.accounts.get_mut(idx) {
                    x.muted = !x.muted;
                }
                a.cfg.save();
                a.invalidate();
            });
        }
        5 => {
            let d = util::profiles_dir().join(&acc.id);
            let _ = std::fs::create_dir_all(&d);
            open_path(&d.to_string_lossy());
        }
        6 => delete_account(idx),
        _ => {}
    }
}

pub(crate) fn tray_menu(x: i32, y: i32) {
    let Some(notify) = with_app(|a| a.cfg.notify) else { return };
    let items = [
        (1, "Mở Betacom", false, true),
        (2, "Thông báo khi có tin nhắn mới", notify, true),
        (0, "", false, true),
        (3, "Thoát Betacom", false, true),
    ];
    match popup_menu(&items, x, y) {
        1 => {
            with_app(|a| a.show_main());
        }
        2 => {
            with_app(|a| {
                a.cfg.notify = !a.cfg.notify;
                a.cfg.save();
            });
        }
        3 => exit_app(),
        _ => {}
    }
}

pub(crate) fn exit_app() {
    let n = with_app(|a| a.sessions.len()).unwrap_or(0);
    if n > 0 {
        let q = format!("Thoát {APP_NAME} sẽ tắt {n} tài khoản Zalo đang chạy.\n\nBạn có muốn thoát không?");
        if msgbox(&q, MB_YESNO | MB_ICONQUESTION) != IDYES {
            return;
        }
    }
    let h = with_app(|a| {
        unsafe { KillTimer(a.hwnd, TIMER_TICK) };
        let ids: Vec<String> = a.sessions.keys().cloned().collect();
        for id in ids {
            if let Some(s) = a.sessions.remove(&id) {
                unsafe {
                    windows_sys::Win32::System::JobObjects::TerminateJobObject(s.job, 0);
                    CloseHandle(s.job);
                }
            }
        }
        a.cfg.save();
        tray::remove(a.hwnd);
        a.hwnd
    });
    if let Some(h) = h {
        unsafe { DestroyWindow(h) };
    }
}

pub(crate) fn do_hit(hit: Hit, screen_pt: POINT) {
    match hit {
        Hit::Account(i) => {
            with_app(|a| a.select(i));
        }
        Hit::Btn(act) => {
            let id = with_app(|a| a.selected_id()).flatten();
            match act {
                Act::AddAcc => add_account(),
                Act::Settings => settings_menu(screen_pt.x, screen_pt.y),
                Act::Download => open_path("https://zalo.me/pc"),
                Act::PickExe => pick_exe(),
                Act::Developer => open_path(DEVELOPER_URL),
                Act::Toggle => {
                    with_app(|a| a.toggle_sidebar());
                }
                Act::Start => {
                    if let Some(id) = id {
                        with_app(|a| {
                            a.failed.remove(&id);
                            a.start(&id)
                        });
                    }
                }
                Act::Stop => {
                    if let Some(id) = id {
                        with_app(|a| a.stop(&id));
                    }
                }
                Act::Restart => {
                    if let Some(id) = id {
                        with_app(|a| a.restart(&id));
                    }
                }
                Act::Reattach => {
                    if let Some(id) = id {
                        with_app(|a| a.reattach(&id));
                    }
                }
            }
        }
        Hit::None => {}
    }
}
