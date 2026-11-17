# Quyết định

- 2026-11-17: Repo là fork của MultiZ v0.3.4 (MIT); được phép dọn cấu trúc kế thừa từ upstream.
- 2026-11-17: Chỉ giữ một `LICENSE` ở gốc; README và PRIVACY chi tiết đặt trong thư mục từng nền tảng, README gốc là trang tổng quan.
- 2026-11-17: Target MSVC biên dịch tài nguyên bằng `rc.exe` của Windows SDK; `windres` chỉ dùng cho target GNU.
- 2026-11-17: Plan gia cố chỉ gồm bản Windows; bản macOS lập plan riêng khi có máy macOS.
- 2026-11-17: Các thay đổi phụ thuộc hành vi Zalo thật (M2, BREAKAWAY_OK, M7/W9, M9, L1, L10, timeout UI Automation) hoãn tới khi kiểm thử được.
- 2026-11-17: Tắt phiên không chờ trên luồng UI; quá 10 giây chưa thoát thì đóng handle, báo lỗi, không mở lại.
- 2026-11-17: Không đọc được `config.json` (khác lỗi không tồn tại) thì chạy chế độ không lưu để tránh ghi đè danh sách tài khoản.
