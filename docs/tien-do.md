# Tiến độ

## Trạng thái

Đã fork MultiZ v0.3.4. Chưa có plan đang làm.

## Nhật ký

### 2026-11-17

- Clone repo, đọc toàn bộ mã nguồn `windows/` và `mac/`.
- Viết `CLAUDE.md`: lệnh build, kiến trúc Windows và macOS.
- Dọn tài liệu trùng: một `LICENSE` ở gốc; README gốc thành trang tổng quan; README và PRIVACY chi tiết nằm trong từng thư mục nền tảng.
- Sửa `windows/README.md`: nêu yêu cầu `windres`, đường dẫn build trong `windows/`.
- Kiểm tra: `cargo fmt --check` đạt; `cargo build --release` dừng ở `build.rs` vì máy thiếu `windres`; bản macOS chưa kiểm chứng (máy không có Swift).

## Việc tiếp theo

- Cài mingw-w64 binutils hoặc đặt `WINDRES`, rồi chạy lại `cargo build --release` và `cargo clippy --release`.
- Thêm `.gitignore` cho `windows/target/`, `mac/build/`.
- Thống nhất hành vi xóa tài khoản giữa hai bản (Windows xóa hẳn, macOS chuyển vào Thùng rác).
- Đánh giá độ bền của `watcher.rs` trước thay đổi bố cục giao diện Zalo.
