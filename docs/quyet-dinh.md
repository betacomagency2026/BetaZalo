# Quyết định

- 2026-11-17: Repo là fork của MultiZ v0.3.4 (MIT); được phép dọn cấu trúc kế thừa từ upstream.
- 2026-11-17: Chỉ giữ một `LICENSE` ở gốc; README và PRIVACY chi tiết đặt trong thư mục từng nền tảng, README gốc là trang tổng quan.
- 2026-11-17: Target MSVC biên dịch tài nguyên bằng `rc.exe` của Windows SDK; `windres` chỉ dùng cho target GNU.
- 2026-11-17: Plan gia cố chỉ gồm bản Windows; bản macOS lập plan riêng khi có máy macOS.
- 2026-11-17: Các thay đổi phụ thuộc hành vi Zalo thật (M2, BREAKAWAY_OK, M7/W9, M9, L1, L10, timeout UI Automation) hoãn tới khi kiểm thử được.
- 2026-11-17: Tắt phiên không chờ trên luồng UI; quá 10 giây chưa thoát thì đóng handle, báo lỗi, không mở lại.
- 2026-11-17: Không đọc được `config.json` (khác lỗi không tồn tại) thì chạy chế độ không lưu để tránh ghi đè danh sách tài khoản.
- 2026-11-17: Đổi tên sản phẩm MultiZ thành Betacom trên cả hai bản; bỏ mục "Mời cà phê".
- 2026-11-17: LICENSE giữ dòng bản quyền Sudo Software bên cạnh Betacom vì MIT bắt buộc giữ thông báo bản quyền gốc.
- 2026-11-17: Liên kết trang chủ và kiểm tra phiên bản trỏ tới repo GitHub `betacomagency2026/BetaZalo` (trang releases).
- 2026-11-17: Định danh mới: exe `betacom.exe`, thư mục dữ liệu `Betacom`, mutex `Local\Betacom.BetaZalo.SingleInstance`, bundle id và LaunchAgent `com.betacom.betazalo`; dữ liệu và mục khởi động của bản MultiZ được tự chuyển sang.
- 2026-11-17: Sửa lỗi macOS M1–M9 ngay theo yêu cầu người dùng dù máy không có Swift; thay cho quyết định lập plan macOS riêng. Phải kiểm chứng bằng `bash build.sh` trên macOS.
- 2026-11-17: macOS xác minh phiên bằng PID kèm thời điểm khởi động tiến trình; chỉ gửi tín hiệu cho tiến trình đã xác minh.
- 2026-11-17: macOS xóa tài khoản khỏi config ngay, chỉ chuyển profile vào Thùng rác sau khi Zalo đã thoát (tối đa 7 giây); chưa thoát thì giữ thư mục và báo người dùng.
- 2026-11-17: Toàn bộ comment trong mã nguồn `windows/src`, `mac/Sources` và `mac/build.sh` được gỡ theo ràng buộc không comment.
