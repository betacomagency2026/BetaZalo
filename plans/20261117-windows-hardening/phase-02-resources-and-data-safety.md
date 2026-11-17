# Phase 2: Rò tài nguyên và an toàn dữ liệu

## Yêu cầu và cách làm

- **H1 rò HICON** (`tray.rs`): nạp icon nhỏ và icon lớn bằng `LR_SHARED` (hệ thống quản lý, không cần `DestroyIcon`), hoặc nạp một lần và giữ trong `OnceLock`. Chọn `LR_SHARED` vì icon lấy từ resource của chính exe.
- **H2/W3 khóa avatar.png** (`gfx.rs::load_image`): đọc file vào bộ nhớ, giải mã qua stream, chép sang bitmap 32bpp độc lập (`GdipCreateBitmapFromScan0` + vẽ lại), giải phóng stream. File không còn bị khóa nên đổi ảnh, bỏ ảnh, xóa tài khoản đều chạy.
- **Thứ tự xóa tài khoản** (`actions.rs::delete_account`): bỏ ảnh khỏi bộ nhớ trước `remove_dir_all`.
- **M8 kiểm tra id** (`actions.rs`, `config.rs`): chỉ xóa khi `id` khớp mẫu `[A-Za-z0-9_-]+`, khác rỗng, và đường dẫn đích nằm trong `profiles_dir()`. Id không hợp lệ thì chỉ gỡ khỏi config, ghi log.
- **M1 config đọc lỗi** (`config.rs::load`): mọi lỗi đọc hoặc parse (trừ không tìm thấy file) đều sao lưu sang `config.broken.json` trước khi dùng mặc định; bản sao lưu cũ được đổi tên theo thời điểm, không bị ghi đè.
- **L6 lưu config** (`config.rs::save`): `sync_all` trước khi đổi tên; ghi log khi lỗi.
- **L7 xoay vòng log** (`util.rs::log`): khóa `Mutex` toàn cục quanh kiểm tra kích thước, đổi tên, ghi.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 1 và 6 trong `plan.md`.

## Rủi ro

- Giải mã từ bộ nhớ cần `SHCreateMemStream` (shlwapi) và feature `Win32_System_Com` của windows-sys; tăng kích thước exe không đáng kể.
- `LR_SHARED` với kích thước khác kích thước gốc vẫn cho icon dùng chung theo cặp (resource, kích thước).
