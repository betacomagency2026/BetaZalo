//! Vẽ giao diện: thanh tài khoản, thanh tiêu đề và khung bên phải.

use super::*;

// màu giao diện (0xRRGGBB)
pub(crate) const C_BG: u32 = 0xFFFFFF;
pub(crate) const C_PANE: u32 = 0xF6F7F9;
pub(crate) const C_LINE: u32 = 0xE6E8EB;
pub(crate) const C_TEXT: u32 = 0x111827;
pub(crate) const C_TEXT2: u32 = 0x6B7280;
pub(crate) const C_MUTED: u32 = 0x9CA3AF;
pub(crate) const C_SEL: u32 = 0xEEF0FF;
pub(crate) const C_HOVER: u32 = 0xF4F5F7;
pub(crate) const C_ACCENT: u32 = 0x4F46E5;
pub(crate) const C_ACCENT_H: u32 = 0x4338CA;
pub(crate) const C_GREEN: u32 = 0x1D9E75;
pub(crate) const C_AMBER: u32 = 0xD97706;
pub(crate) const C_RED: u32 = 0xE5484D;
pub(crate) const C_BORDER: u32 = 0xD0D5DD;

pub(crate) struct Fonts {
    pub(crate) logo: HFONT,
    pub(crate) name: HFONT,
    pub(crate) small: HFONT,
    pub(crate) badge: HFONT,
    pub(crate) btn: HFONT,
    pub(crate) h1: HFONT,
    pub(crate) body: HFONT,
    pub(crate) avatar: HFONT,
}

impl Fonts {
    pub(crate) fn new(sc: f32) -> Fonts {
        let px = |v: f32| (v * sc).round() as i32;
        Fonts {
            logo: make_font(px(17.0), 700, "Segoe UI"),
            name: make_font(px(14.0), 600, "Segoe UI"),
            small: make_font(px(12.0), 400, "Segoe UI"),
            badge: make_font(px(11.0), 600, "Segoe UI"),
            btn: make_font(px(13.0), 600, "Segoe UI"),
            h1: make_font(px(20.0), 600, "Segoe UI"),
            body: make_font(px(14.0), 400, "Segoe UI"),
            avatar: make_font(px(13.0), 600, "Segoe UI"),
        }
    }
    pub(crate) fn free(&self) {
        for f in [self.logo, self.name, self.small, self.badge, self.btn, self.h1, self.body, self.avatar] {
            unsafe { DeleteObject(f as _) };
        }
    }
}

impl App {
    pub(crate) fn button(
        &mut self,
        cv: &mut Canvas,
        x: i32,
        y: i32,
        label: &str,
        act: Act,
        primary: bool,
        in_pane: bool,
    ) -> i32 {
        let h = self.s(34);
        let wd = text_width(cv.hdc, label, self.fonts.btn) + self.s(32);
        let r = rc(x, y, x + wd, y + h);
        let hov = if in_pane { self.pane_hover } else { self.hover } == Hit::Btn(act);
        let rad = self.s(8);
        if primary {
            cv.fill_round(x, y, wd, h, rad, if hov { C_ACCENT_H } else { C_ACCENT });
            cv.text(r, label, self.fonts.btn, 0xFFFFFF, DT_SINGLELINE | DT_VCENTER | DT_CENTER);
        } else {
            cv.fill_round(x, y, wd, h, rad, if hov { C_HOVER } else { 0xFFFFFF });
            cv.stroke_round(x, y, wd, h, rad, C_BORDER, 1.0);
            cv.text(r, label, self.fonts.btn, 0x1F2937, DT_SINGLELINE | DT_VCENTER | DT_CENTER);
        }
        if in_pane {
            self.pane_hits.push((r, Hit::Btn(act)));
        } else {
            self.hits.push((r, Hit::Btn(act)));
        }
        wd
    }

    /// Biểu tượng ly cà phê vẽ tay (không phụ thuộc font emoji của máy).
    pub(crate) fn draw_cup(&self, cv: &mut Canvas, x: i32, y: i32, size: i32, color: u32) {
        let u = size as f32 / 16.0;
        let p = |v: f32| (v * u).round() as i32;
        // quai
        cv.stroke_round(x + p(8.5), y + p(5.5), p(6.0), p(6.0), p(3.0), color, (1.6 * u).max(1.0));
        // thân cốc
        cv.fill_round(x + p(1.0), y + p(4.0), p(10.5), p(9.0), p(2.5), color);
        // đĩa
        cv.fill_round(x, y + p(14.0), p(13.5), p(2.0), p(1.0), color);
        // khói
        cv.fill_round(x + p(3.5), y, p(1.6), p(3.0), p(1.0), color);
        cv.fill_round(x + p(7.0), y, p(1.6), p(3.0), p(1.0), color);
    }

    /// Ảnh đại diện: ảnh người dùng chọn, nếu không có thì chữ viết tắt trên nền màu.
    pub(crate) fn draw_avatar(&self, cv: &mut Canvas, a: &Account, x: i32, y: i32, d: i32, off: bool) {
        if let Some(img) = self.avatars.get(&a.id) {
            cv.image_round(img, x, y, d, d, d / 2);
            return;
        }
        let (bg, fg) = COLORS[a.color % COLORS.len()];
        cv.fill_circle(x, y, d, if off { 0xF1F2F4 } else { bg });
        cv.text(
            rc(x, y, x + d, y + d),
            &initials(&a.name),
            self.fonts.avatar,
            if off { C_MUTED } else { fg },
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
    }

    pub(crate) fn status_of(&self, id: &str) -> (String, u32, u32) {
        // (chữ trạng thái, màu chấm, màu chữ)
        if self.has_new.contains(id) {
            return ("Có tin nhắn mới".into(), C_GREEN, C_RED);
        }
        let (t, c) = match self.sessions.get(id) {
            Some(s) => match (&s.state, s.hwnd.is_null()) {
                (State::Running, false) => ("Đang chạy", C_GREEN),
                _ => ("Đang mở…", C_AMBER),
            },
            None if self.failed.contains_key(id) => ("Lỗi khi mở", C_RED),
            None if self.queue.iter().any(|q| q == id) => ("Chờ mở…", C_MUTED),
            None => ("Đã tắt", C_MUTED),
        };
        let muted = self.account(id).map(|a| a.muted).unwrap_or(false);
        let t = if muted && self.sessions.contains_key(id) { format!("{t} · Tắt báo") } else { t.to_string() };
        (t, c, C_TEXT2)
    }

    pub(crate) fn paint_main(&mut self, hdc: HDC) {
        let mut cr: RECT = unsafe { std::mem::zeroed() };
        unsafe { GetClientRect(self.hwnd, &mut cr) };
        self.hits.clear();
        let mut cv = Canvas::new(hdc);

        if self.cfg.sidebar_collapsed {
            self.paint_sidebar_collapsed(&mut cv, &cr);
        } else {
            self.paint_sidebar_full(&mut cv, &cr, hdc);
        }
        let sel = self.selected_index();

        // ---- thanh tiêu đề bên phải ----
        self.paint_header(&mut cv, &cr, hdc, sel);
        cv.finish();
    }

    pub(crate) fn paint_sidebar_collapsed(&mut self, cv: &mut Canvas, cr: &RECT) {
        let sw = self.sidebar_w();
        cv.fill_rect(rc(0, 0, sw, cr.bottom), C_BG);
        cv.fill_rect(rc(sw - 1, 0, sw, cr.bottom), C_LINE);
        let lg = self.s(30);
        let (lx, ly) = ((sw - lg) / 2, self.s(14));
        cv.fill_round(lx, ly, lg, lg, self.s(8), C_ACCENT);
        cv.fill_round(lx + self.s(10), ly + self.s(7), self.s(14), self.s(10), self.s(5), 0xA5B4FC);
        cv.fill_round(lx + self.s(6), ly + self.s(12), self.s(15), self.s(11), self.s(5), 0xFFFFFF);
        // nút mở rộng
        let (tw, th) = (self.s(36), self.s(26));
        let (tx, ty) = ((sw - tw) / 2, self.s(52));
        if self.hover == Hit::Btn(Act::Toggle) {
            cv.fill_round(tx, ty, tw, th, self.s(6), C_HOVER);
        }
        cv.text(
            rc(tx, ty - self.s(1), tx + tw, ty + th - self.s(1)),
            "»",
            self.fonts.h1,
            C_TEXT2,
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
        self.hits.push((rc(tx, ty, tx + tw, ty + th), Hit::Btn(Act::Toggle)));

        let n = self.cfg.accounts.len();
        let item_h = self.s(56);
        let list_top = self.s(90);
        let list_bottom = cr.bottom - self.s(118);
        self.list_rect = rc(0, list_top, sw, list_bottom);
        let cap = (((list_bottom - list_top) / item_h).max(1)) as usize;
        if self.scroll + cap > n {
            self.scroll = n.saturating_sub(cap);
        }
        let sel = self.selected_index();
        let accounts: Vec<Account> = self.cfg.accounts.clone();
        let mut y = list_top;
        for (i, a) in accounts.iter().enumerate().skip(self.scroll).take(cap) {
            let r = rc(self.s(8), y, sw - self.s(8), y + item_h - self.s(4));
            if sel == Some(i) {
                cv.fill_round(r.left, r.top, r.right - r.left, r.bottom - r.top, self.s(10), C_SEL);
            } else if self.hover == Hit::Account(i) {
                cv.fill_round(r.left, r.top, r.right - r.left, r.bottom - r.top, self.s(10), C_HOVER);
            }
            let av = self.s(38);
            let ax = (sw - av) / 2;
            let ay = r.top + (r.bottom - r.top - av) / 2;
            let (status, scol, _) = self.status_of(&a.id);
            let off = status == "Đã tắt";
            self.draw_avatar(cv, a, ax, ay, av, off);
            let ring = if sel == Some(i) { C_SEL } else { 0xFFFFFF };
            // chấm trạng thái (góc dưới phải)
            let dd = self.s(12);
            cv.fill_circle(ax + av - dd + self.s(2), ay + av - dd + self.s(2), dd, ring);
            cv.fill_circle(ax + av - dd + self.s(4), ay + av - dd + self.s(4), dd - self.s(4), scol);
            // số chưa đọc / chấm đỏ tin mới (góc trên phải)
            let unread = self.sessions.get(&a.id).map(|s| s.unread).unwrap_or(0);
            if unread > 0 {
                let txt = if unread > 99 { "99+".to_string() } else { unread.to_string() };
                let bh = self.s(18);
                let bw = (text_width(cv.hdc, &txt, self.fonts.badge) + self.s(10)).max(bh);
                let bx = ax + av - bw / 2 - self.s(2);
                let by = ay - self.s(5);
                cv.fill_round(
                    bx - self.s(2),
                    by - self.s(2),
                    bw + self.s(4),
                    bh + self.s(4),
                    (bh + self.s(4)) / 2,
                    ring,
                );
                cv.fill_round(bx, by, bw, bh, bh / 2, C_RED);
                cv.text(
                    rc(bx, by, bx + bw, by + bh),
                    &txt,
                    self.fonts.badge,
                    0xFFFFFF,
                    DT_SINGLELINE | DT_VCENTER | DT_CENTER,
                );
            } else if self.has_new.contains(&a.id) {
                let d = self.s(12);
                let dx = ax + av - d + self.s(1);
                let dy = ay - self.s(1);
                cv.fill_circle(dx - self.s(2), dy - self.s(2), d + self.s(4), ring);
                cv.fill_circle(dx, dy, d, C_RED);
            }
            self.hits.push((r, Hit::Account(i)));
            y += item_h;
        }
        if self.scroll + cap < n {
            cv.text(
                rc(0, list_bottom - self.s(4), sw, list_bottom + self.s(12)),
                "▼",
                self.fonts.small,
                C_MUTED,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }
        // nút cà phê + nút thêm tài khoản (hình tròn)
        let d = self.s(40);
        let (bx, by) = ((sw - d) / 2, cr.bottom - self.s(60));
        let cy = by - d - self.s(8);
        let cafe_hov = self.hover == Hit::Btn(Act::Coffee);
        cv.fill_circle(bx, cy, d, if cafe_hov { 0xFFEDD5 } else { 0xFFF7ED });
        cv.stroke_round(bx, cy, d, d, d / 2, 0xFED7AA, 1.0);
        let cup = self.s(18);
        self.draw_cup(cv, bx + (d - cup) / 2 + self.s(1), cy + (d - cup) / 2, cup, 0x9A3412);
        self.hits.push((rc(bx, cy, bx + d, cy + d), Hit::Btn(Act::Coffee)));
        let hov = self.hover == Hit::Btn(Act::AddAcc);
        cv.fill_circle(bx, by, d, if hov { C_HOVER } else { 0xFFFFFF });
        cv.stroke_round(bx, by, d, d, d / 2, C_BORDER, 1.0);
        cv.text(
            rc(bx, by - self.s(2), bx + d, by + d - self.s(2)),
            "+",
            self.fonts.h1,
            0x1F2937,
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
        self.hits.push((rc(bx, by, bx + d, by + d), Hit::Btn(Act::AddAcc)));
    }

    pub(crate) fn paint_sidebar_full(&mut self, cv: &mut Canvas, cr: &RECT, hdc: HDC) {
        let cr = *cr;
        let sw = self.sidebar_w();
        // ---- thanh bên ----
        cv.fill_rect(rc(0, 0, sw, cr.bottom), C_BG);
        cv.fill_rect(rc(sw - 1, 0, sw, cr.bottom), C_LINE);
        let lg = self.s(30);
        let (lx, ly) = (self.s(18), self.s(14));
        cv.fill_round(lx, ly, lg, lg, self.s(8), C_ACCENT);
        cv.fill_round(lx + self.s(10), ly + self.s(7), self.s(14), self.s(10), self.s(5), 0xA5B4FC);
        cv.fill_round(lx + self.s(6), ly + self.s(12), self.s(15), self.s(11), self.s(5), 0xFFFFFF);
        cv.text(
            rc(lx + lg + self.s(10), ly, sw, ly + lg),
            APP_NAME,
            self.fonts.logo,
            C_TEXT,
            DT_SINGLELINE | DT_VCENTER,
        );
        // nút thu gọn
        let (tw, th) = (self.s(32), self.s(28));
        let (tx, ty) = (sw - tw - self.s(12), ly + (lg - th) / 2);
        if self.hover == Hit::Btn(Act::Toggle) {
            cv.fill_round(tx, ty, tw, th, self.s(6), C_HOVER);
        }
        cv.text(
            rc(tx, ty - self.s(1), tx + tw, ty + th - self.s(1)),
            "«",
            self.fonts.h1,
            C_TEXT2,
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
        self.hits.push((rc(tx, ty, tx + tw, ty + th), Hit::Btn(Act::Toggle)));

        let n = self.cfg.accounts.len();
        let y0 = self.s(62);
        let new_n = self.has_new.len();
        let head = if new_n > 0 {
            format!("Tài khoản ({n}) · {new_n} có tin mới")
        } else {
            format!("Tài khoản ({n})")
        };
        cv.text(
            rc(self.s(20), y0, sw - self.s(16), y0 + self.s(22)),
            &head,
            self.fonts.small,
            C_TEXT2,
            DT_SINGLELINE | DT_VCENTER,
        );

        // phần đáy: chân trang, nút thêm tài khoản, nút cà phê
        let b = cr.bottom;
        let foot1 = rc(0, b - self.s(46), sw, b - self.s(28));
        let foot2 = rc(0, b - self.s(27), sw, b - self.s(8));
        let add_h = self.s(40);
        let add_y = b - self.s(98);
        let cafe_h = self.s(34);
        let cafe_y = add_y - self.s(8) - cafe_h;
        let list_bottom = cafe_y - self.s(10);

        // ---- danh sách tài khoản (có cuộn) ----
        let item_h = self.s(56);
        let list_top = y0 + self.s(28);
        self.list_rect = rc(0, list_top, sw, list_bottom);
        let cap = (((list_bottom - list_top) / item_h).max(1)) as usize;
        if self.scroll + cap > n {
            self.scroll = n.saturating_sub(cap);
        }
        let mut y = list_top;
        let sel = self.selected_index();
        let accounts: Vec<Account> = self.cfg.accounts.clone();
        for (i, a) in accounts.iter().enumerate().skip(self.scroll).take(cap) {
            let r = rc(self.s(10), y, sw - self.s(10), y + item_h - self.s(4));
            let bg = if sel == Some(i) {
                Some(C_SEL)
            } else if self.hover == Hit::Account(i) {
                Some(C_HOVER)
            } else {
                None
            };
            if let Some(bg) = bg {
                cv.fill_round(r.left, r.top, r.right - r.left, r.bottom - r.top, self.s(10), bg);
            }
            let av = self.s(36);
            let ax = r.left + self.s(10);
            let ay = r.top + (r.bottom - r.top - av) / 2;
            let (status, scol, tcol) = self.status_of(&a.id);
            let off = status == "Đã tắt";
            self.draw_avatar(cv, a, ax, ay, av, off);
            let dd = self.s(11);
            cv.fill_circle(
                ax + av - dd + self.s(2),
                ay + av - dd + self.s(2),
                dd,
                if sel == Some(i) { C_SEL } else { 0xFFFFFF },
            );
            cv.fill_circle(ax + av - dd + self.s(4), ay + av - dd + self.s(4), dd - self.s(4), scol);

            let unread = self.sessions.get(&a.id).map(|s| s.unread).unwrap_or(0);
            let mut tx_right = r.right - self.s(10);
            if unread > 0 {
                let txt = if unread > 99 { "99+".to_string() } else { unread.to_string() };
                let bw = (text_width(cv.hdc, &txt, self.fonts.badge) + self.s(12)).max(self.s(20));
                let bh = self.s(20);
                let bx = r.right - self.s(10) - bw;
                let by = r.top + (r.bottom - r.top - bh) / 2;
                cv.fill_round(bx, by, bw, bh, bh / 2, C_RED);
                cv.text(
                    rc(bx, by, bx + bw, by + bh),
                    &txt,
                    self.fonts.badge,
                    0xFFFFFF,
                    DT_SINGLELINE | DT_VCENTER | DT_CENTER,
                );
                tx_right = bx - self.s(6);
            } else if self.has_new.contains(&a.id) {
                let d = self.s(10);
                let dx = r.right - self.s(14) - d;
                let dy = r.top + (r.bottom - r.top - d) / 2;
                cv.fill_circle(dx, dy, d, C_RED);
                tx_right = dx - self.s(8);
            }
            let tx = ax + av + self.s(12);
            let mid = r.top + (r.bottom - r.top) / 2;
            let name_col = if sel == Some(i) {
                0x3730A3
            } else if off {
                C_TEXT2
            } else {
                C_TEXT
            };
            cv.text(
                rc(tx, mid - self.s(19), tx_right, mid + self.s(1)),
                &a.name,
                self.fonts.name,
                name_col,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            cv.text(
                rc(tx, mid + self.s(1), tx_right, mid + self.s(19)),
                &status,
                self.fonts.small,
                tcol,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            self.hits.push((r, Hit::Account(i)));
            y += item_h;
        }
        if n > cap {
            // gợi ý còn tài khoản khi cuộn
            let more_up = self.scroll > 0;
            let more_down = self.scroll + cap < n;
            let hint = match (more_up, more_down) {
                (true, true) => "▲ ▼ Cuộn để xem thêm",
                (false, true) => "▼ Cuộn để xem thêm",
                _ => "▲ Cuộn lên",
            };
            cv.text(
                rc(0, list_bottom - self.s(2), sw, list_bottom + self.s(12)),
                hint,
                self.fonts.small,
                C_MUTED,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }

        // nút mời cà phê (mở trang ủng hộ trên trình duyệt)
        {
            let x = self.s(14);
            let wd = sw - self.s(28);
            let hov = self.hover == Hit::Btn(Act::Coffee);
            cv.fill_round(x, cafe_y, wd, cafe_h, self.s(10), if hov { 0xFFEDD5 } else { 0xFFF7ED });
            cv.stroke_round(x, cafe_y, wd, cafe_h, self.s(10), 0xFED7AA, 1.0);
            let label = "Mời MultiZ ly cà phê";
            let cup = self.s(16);
            let gap = self.s(8);
            let tw = text_width(cv.hdc, label, self.fonts.btn);
            let start = x + (wd - (cup + gap + tw)) / 2;
            self.draw_cup(cv, start, cafe_y + (cafe_h - cup) / 2, cup, 0x9A3412);
            let text_rc = rc(start + cup + gap, cafe_y, x + wd, cafe_y + cafe_h);
            cv.text(text_rc, label, self.fonts.btn, 0x9A3412, DT_SINGLELINE | DT_VCENTER);
            self.hits.push((rc(x, cafe_y, x + wd, cafe_y + cafe_h), Hit::Btn(Act::Coffee)));
        }

        // nút thêm tài khoản + chân trang
        let bx = self.s(14);
        let bw = sw - self.s(28);
        let hov = self.hover == Hit::Btn(Act::AddAcc);
        cv.fill_round(bx, add_y, bw, add_h, self.s(10), if hov { C_HOVER } else { 0xFFFFFF });
        cv.stroke_round(bx, add_y, bw, add_h, self.s(10), C_BORDER, 1.0);
        cv.text(
            rc(bx, add_y, bx + bw, add_y + add_h),
            "+   Thêm tài khoản",
            self.fonts.btn,
            0x1F2937,
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
        self.hits.push((rc(bx, add_y, bx + bw, add_y + add_h), Hit::Btn(Act::AddAcc)));
        let rc_n = self.sessions.len();
        let f1 = if rc_n == 0 {
            format!("{APP_NAME} {APP_VERSION} · chưa có tài khoản nào chạy")
        } else {
            format!("{APP_NAME} {APP_VERSION} · {rc_n} tài khoản đang chạy")
        };
        cv.text(foot1, &f1, self.fonts.small, C_MUTED, DT_SINGLELINE | DT_VCENTER | DT_CENTER);
        let link_hov = self.hover == Hit::Btn(Act::Developer);
        let f2 = format!("Phát triển bởi {DEVELOPER}");
        cv.text(
            foot2,
            &f2,
            self.fonts.small,
            if link_hov { C_ACCENT_H } else { C_ACCENT },
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
        let fw = text_width(hdc, &f2, self.fonts.small);
        self.hits.push((rc((sw - fw) / 2, foot2.top, (sw + fw) / 2, foot2.bottom), Hit::Btn(Act::Developer)));
    }

    pub(crate) fn paint_header(&mut self, cv: &mut Canvas, cr: &RECT, hdc: HDC, sel: Option<usize>) {
        let cr = *cr;
        let sw = self.sidebar_w();
        let hh = self.header_h();
        cv.fill_rect(rc(sw, 0, cr.right, hh), C_BG);
        cv.fill_rect(rc(sw, hh - 1, cr.right, hh), C_LINE);
        let by = (hh - self.s(34)) / 2;
        let mut x = cr.right - self.s(16);
        let settings_w = text_width(hdc, "Cài đặt", self.fonts.btn) + self.s(32);
        x -= settings_w;
        self.button(cv, x, by, "Cài đặt", Act::Settings, false, false);
        if let Some(i) = sel {
            let a = self.cfg.accounts[i].clone();
            let running = self.sessions.contains_key(&a.id);
            let labels: Vec<(&str, Act, bool)> = if running {
                vec![
                    ("Tắt", Act::Stop, false),
                    ("Khởi động lại", Act::Restart, false),
                    ("Gắn lại cửa sổ", Act::Reattach, false),
                ]
            } else {
                vec![("Mở Zalo", Act::Start, false)]
            };
            for (label, act, primary) in labels {
                let wd = text_width(hdc, label, self.fonts.btn) + self.s(32);
                x -= wd + self.s(8);
                self.button(cv, x, by, label, act, primary, false);
            }
            let tx = sw + self.s(20);
            let name_w = text_width(hdc, &a.name, self.fonts.name).min(x - tx - self.s(150)).max(self.s(40));
            cv.text(
                rc(tx, 0, tx + name_w, hh),
                &a.name,
                self.fonts.name,
                C_TEXT,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            let pill = "Dữ liệu riêng";
            let pw = text_width(hdc, pill, self.fonts.small) + self.s(18);
            let px = tx + name_w + self.s(10);
            let ph = self.s(22);
            if px + pw < x - self.s(8) {
                cv.fill_round(px, (hh - ph) / 2, pw, ph, ph / 2, 0xE7F6EE);
                cv.text(
                    rc(px, (hh - ph) / 2, px + pw, (hh + ph) / 2),
                    pill,
                    self.fonts.small,
                    0x0F6E56,
                    DT_SINGLELINE | DT_VCENTER | DT_CENTER,
                );
            }
        }
    }

    pub(crate) fn paint_pane(&mut self, hdc: HDC) {
        let mut cr: RECT = unsafe { std::mem::zeroed() };
        unsafe { GetClientRect(self.pane, &mut cr) };
        self.pane_hits.clear();
        let mut cv = Canvas::new(hdc);
        cv.fill_rect(cr, C_PANE);

        let sel = self.selected_index();
        let (title, body, buttons): (String, String, Vec<(&str, Act, bool)>) = if self.zalo.is_none() {
            (
                "Chưa tìm thấy Zalo PC trên máy".into(),
                format!("{APP_NAME} dùng chính ứng dụng Zalo PC gốc. Hãy cài Zalo PC, hoặc chỉ cho {APP_NAME} biết vị trí file Zalo.exe."),
                vec![("Tải Zalo PC", Act::Download, true), ("Chọn Zalo.exe…", Act::PickExe, false)],
            )
        } else if self.cfg.accounts.is_empty() {
            (
                format!("Chào mừng bạn đến với {APP_NAME}"),
                "Mỗi tài khoản Zalo chạy trong một vùng dữ liệu riêng, không lẫn vào nhau. Thêm tài khoản đầu tiên rồi quét mã QR để đăng nhập.".into(),
                vec![("Thêm tài khoản", Act::AddAcc, true)],
            )
        } else if let Some(i) = sel {
            let a = &self.cfg.accounts[i];
            match self.sessions.get(&a.id) {
                None => {
                    if let Some(err) = self.failed.get(&a.id) {
                        ("Không mở được Zalo".into(), err.clone(), vec![("Thử lại", Act::Start, true)])
                    } else {
                        (
                            format!("{} đang tắt", a.name),
                            "Mở Zalo để tiếp tục nhận và gửi tin nhắn cho tài khoản này.".into(),
                            vec![("Mở Zalo", Act::Start, true)],
                        )
                    }
                }
                Some(s) if s.hwnd.is_null() => {
                    if s.started.elapsed().as_secs() < 20 {
                        (format!("Đang mở Zalo cho {}…", a.name), "Lần đầu mở có thể mất vài giây. Nếu là tài khoản mới, hãy quét mã QR bằng ứng dụng Zalo trên điện thoại.".into(), vec![])
                    } else {
                        (
                            "Chưa thấy cửa sổ Zalo".into(),
                            "Zalo có thể đang chạy ẩn dưới khay hệ thống (góc phải thanh Taskbar). Bấm \"Gắn lại cửa sổ\" để đưa Zalo vào đây.".into(),
                            vec![("Gắn lại cửa sổ", Act::Reattach, true), ("Khởi động lại", Act::Restart, false)],
                        )
                    }
                }
                Some(_) => {
                    if self.cfg.embed {
                        (String::new(), String::new(), vec![])
                    } else {
                        (
                            "Zalo đang hiển thị ở chế độ cửa sổ gắn kèm".into(),
                            format!("Cửa sổ Zalo nằm đè đúng lên khu vực này và di chuyển theo {APP_NAME}."),
                            vec![],
                        )
                    }
                }
            }
        } else {
            ("Chọn một tài khoản ở bên trái".into(), String::new(), vec![])
        };

        if !title.is_empty() {
            let cw = cr.right - cr.left;
            let box_w = self.s(480).min(cw - self.s(40));
            let bx = (cw - box_w) / 2;
            let mut y = cr.bottom / 2 - self.s(110);
            let ic = self.s(56);
            let ix = (cw - ic) / 2;
            cv.fill_round(ix, y, ic, ic, self.s(16), 0xEEF0FF);
            cv.fill_round(ix + self.s(20), y + self.s(14), self.s(24), self.s(17), self.s(8), 0xA5B4FC);
            cv.fill_round(ix + self.s(12), y + self.s(23), self.s(26), self.s(19), self.s(9), C_ACCENT);
            y += ic + self.s(18);
            cv.text(
                rc(bx, y, bx + box_w, y + self.s(30)),
                &title,
                self.fonts.h1,
                C_TEXT,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER | DT_END_ELLIPSIS,
            );
            y += self.s(38);
            if !body.is_empty() {
                let mut mr = rc(bx, y, bx + box_w, y + 1000);
                unsafe {
                    let old = SelectObject(hdc, self.fonts.body as _);
                    let wb = w(&body);
                    DrawTextW(hdc, wb.as_ptr(), -1, &mut mr, DT_CALCRECT | DT_WORDBREAK | DT_CENTER | DT_NOPREFIX);
                    SelectObject(hdc, old);
                }
                let bh = mr.bottom - mr.top;
                cv.text(rc(bx, y, bx + box_w, y + bh), &body, self.fonts.body, C_TEXT2, DT_WORDBREAK | DT_CENTER);
                y += bh + self.s(22);
            }
            if !buttons.is_empty() {
                let gap = self.s(10);
                let total: i32 =
                    buttons.iter().map(|(l, _, _)| text_width(hdc, l, self.fonts.btn) + self.s(32)).sum::<i32>()
                        + gap * (buttons.len() as i32 - 1);
                let mut x = (cw - total) / 2;
                for (l, a, p) in buttons {
                    let wd = self.button(&mut cv, x, y, l, a, p, true);
                    x += wd + gap;
                }
            }
        }
        cv.finish();
    }
}
