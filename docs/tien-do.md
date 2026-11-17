# Tiến độ

## Trạng thái

Plan `plans/20261117-windows-hardening`: xong mã cả 6 phase; build, clippy, fmt đạt; chưa kiểm thử tay với Zalo thật. Đã đổi tên sản phẩm thành Betacom.

## Nhật ký

### 2026-11-17

- Clone repo, đọc toàn bộ mã nguồn `windows/` và `mac/`.
- Viết `CLAUDE.md`: lệnh build, kiến trúc Windows và macOS.
- Dọn tài liệu trùng: một `LICENSE` ở gốc; README gốc thành trang tổng quan; README và PRIVACY chi tiết nằm trong từng thư mục nền tảng.
- Audit bản Windows (độ đúng, hiệu năng) và bản macOS; lập plan gia cố 6 phase; red-team rồi sửa plan.
- Phase 1: `build.rs` dùng `rc.exe` cho target MSVC (`windres` cho GNU); sửa 8 cảnh báo clippy.
- Phase 2: nạp icon khay một lần; chép ảnh đại diện vào bộ nhớ để nhả khóa file; config đọc lỗi thì chạy chế độ không lưu, sao lưu file hỏng theo thời điểm, lưu có `sync_all`; khóa log; xóa tài khoản kiểm tra id, thử lại và báo đường dẫn còn sót.
- Phase 3: hoãn bố cục khi window procedure bị gọi lồng (`WM_APP_RELAYOUT`); bỏ qua thu nhỏ; trao focus khi mở từ khay; xóa cờ khay qua `WM_SHOWWINDOW`; tìm lại Zalo.exe khi đường dẫn cũ mất; dừng tiến trình nếu không gán được vào Job Object; xóa trạng thái tin mới khi phiên thoát.
- Phase 4: tắt phiên không chờ trên luồng UI (`stopping`, thời hạn 10 giây, trạng thái "Đang tắt…"); khởi động lại chỉ mở sau khi phiên cũ thoát; bỏ qua cửa sổ đứng hình.
- Phase 5: watcher tạo truy vấn UI Automation một lần; bỏ tiêu đề cửa sổ Zalo khỏi log.
- Phase 6: cắt chuỗi khay không tách cặp surrogate; Enter trên nút Hủy đóng hộp thoại như Hủy.
- Kiểm tra: `cargo build --release`, `cargo clippy --release` không cảnh báo, `cargo fmt --check` đạt. Bản macOS chưa kiểm chứng (máy không có Swift).
- Đổi tên MultiZ thành Betacom (mã, tài nguyên exe, bundle macOS, tài liệu, LICENSE); bỏ nút và mục menu "Mời cà phê"; chuyển dữ liệu, mục khởi động cùng Windows và LaunchAgent của bản MultiZ sang tên mới. Build, clippy, fmt bản Windows đạt; bản macOS chưa biên dịch thử.

## Việc tiếp theo

- Kiểm thử tay việc đổi tên: thoát bản MultiZ, mở `betacom.exe`; dữ liệu `%LOCALAPPDATA%\MultiZ` chuyển sang `Betacom`, tài khoản giữ đăng nhập, thanh bên không còn nút cà phê.
- Biên dịch thử bản macOS sau khi đổi tên (bundle id mới, LaunchAgent mới, chuyển thư mục `MultiZ`).
- Cung cấp website chính thức của Betacom nếu có, thay liên kết GitHub ở "Trang chủ Betacom".
- Chạy 7 mục kiểm thử tay trong `plans/20261117-windows-hardening/plan.md` trên máy có Zalo.
- Các mục hoãn cần thử với Zalo thật: M2 (ẩn cửa sổ khi xuống khay, chế độ gắn kèm), bỏ `BREAKAWAY_OK`, đóng Zalo êm khi tắt máy (M7, W9), Folder Redirection (L10), chọn nhầm cửa sổ phụ (M9), menu khay mở hai lần (L1).
- Timeout UI Automation: cần `CUIAutomation8` và số liệu thời gian đọc p50/p95.
- Lập plan riêng cho bản macOS (cần máy macOS) dựa trên `reports/audit-performance-mac.md`.
- Thống nhất hành vi xóa tài khoản giữa hai bản (Windows xóa hẳn, macOS chuyển vào Thùng rác).
- Đánh giá độ bền của `watcher.rs` trước thay đổi bố cục giao diện Zalo.
- Lỗi mới (rà soát sau đổi tên), theo mức độ:
  - Trung bình: mutex mới không chặn bản MultiZ cũ đang chạy; hai bản cùng ghi `config.json` hoặc chuyển thư mục thất bại (`main.rs` mutex, `util::data_root`). Cần kiểm tra mutex `Local\SudoSoftware.MultiZ.SingleInstance` hoặc class `MultiZMain`, yêu cầu thoát bản cũ; ghi log khi bỏ qua chuyển dữ liệu.
  - Trung bình: `delete_account` chờ thoát, ngủ và `remove_dir_all` trên luồng UI, cập nhật config sau cùng; cần xóa khỏi config trước, đổi tên thư mục sang `.trash-<id>`, xóa ở luồng phụ, dọn khi khởi động.
  - Thấp: `migrate_autorun` bật lại mục khởi động đã tắt trong Task Manager (`StartupApproved\Run`).
  - Thấp: chế độ `read_only` không báo người dùng; xóa tài khoản vẫn xóa thư mục dù không lưu được config.
  - Thấp: `select()` gọi `save()` có `sync_all` mỗi lần bấm tài khoản.
  - Thấp: `UPDATE_URL` trỏ tới trang releases còn trống; `windows/PRIVACY.md` hứa `SHA256SUMS.txt` nhưng chưa có quy trình phát hành.
  - Thấp: còn khoảng 96 dòng comment kế thừa trong `windows/src`, trái ràng buộc không comment.
- Kiểm chứng khi thử tay: Zalo có lưu đường dẫn tuyệt đối (thư mục tải, tệp cục bộ) trong profile không; nếu có, tệp cũ mất sau khi đổi `MultiZ` thành `Betacom`.
- Hướng phát triển: CI GitHub Actions (build, clippy, fmt) và phát hành `betacom.exe` kèm `SHA256SUMS.txt`; phím tắt Ctrl+1..9, nhớ vị trí cửa sổ, toast kích hoạt đúng tài khoản (W7, W10); plan macOS M1–M9.
