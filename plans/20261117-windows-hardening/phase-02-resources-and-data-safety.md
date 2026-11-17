# Phase 2: Rò tài nguyên và an toàn dữ liệu

## Yêu cầu và cách làm

- **H1 rò HICON** (`tray.rs`): nạp icon nhỏ (SM_CXSMICON) và icon lớn (SM_CXICON) một lần, giữ trong `thread_local! Cell<HICON>` (luồng UI); không dùng `LR_SHARED` vì cờ này bỏ qua kích thước yêu cầu khi icon đã có trong cache.
- **H2/W3 khóa avatar.png** (`gfx.rs::load_image`): mở bằng `GdipCreateBitmapFromFile`, chép sang bitmap 32bpp ARGB tạo bằng `GdipCreateBitmapFromScan0`, rồi dispose bitmap nguồn để nhả khóa file.
- **Xóa tài khoản** (`actions.rs::delete_account`):
  - Bỏ ảnh khỏi bộ nhớ trước `remove_dir_all`.
  - **M8**: chỉ xóa thư mục khi `id` khớp `[A-Za-z0-9_-]+`, không rỗng, và không trùng `id` tài khoản khác; ngược lại chỉ gỡ khỏi config và ghi log.
  - Khi `remove_dir_all` lỗi: thử lại sau 1 giây; vẫn lỗi thì báo người dùng đường dẫn còn sót để xóa tay, vẫn gỡ tài khoản khỏi danh sách (người dùng đã xác nhận xóa).
- **M1 config đọc lỗi** (`config.rs::load`):
  - Không tìm thấy file: mặc định.
  - Lỗi đọc khác (bị khóa, quyền): thử lại 3 lần cách 200 ms; vẫn lỗi thì dùng mặc định và đặt `read_only = true` (trường `#[serde(skip)]`), `save()` không ghi, ghi log.
  - Lỗi parse: sao lưu sang `config.broken-<thời điểm>.json`; sao lưu lỗi thì cũng đặt `read_only`.
- **L6 lưu config** (`config.rs::save`): ghi qua `File`, `sync_all`, rồi đổi tên; ghi log khi lỗi.
- **L7 xoay vòng log** (`util.rs::log`): `Mutex` toàn cục quanh kiểm tra kích thước, đổi tên, ghi.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 1, 6, 7 trong `plan.md`.
