# Phase 3: Gọi lồng và trạng thái cửa sổ

## Yêu cầu và cách làm

- **H3 bố cục bị bỏ** (`main.rs` WM_SIZE): khi `with_app` trả `None` thì `PostMessageW(WM_APP_RELAYOUT)`; thông điệp này gọi lại `layout`. Áp dụng cùng mẫu cho WM_DPICHANGED (đổi font) và WM_MOVE/WM_WINDOWPOSCHANGED.
- **L2 focus khi mở từ khay** (`show_main`): gửi `WM_APP_FOCUS` sau khi hiện cửa sổ.
- **M4 cờ hidden_to_tray** (`main.rs`): xử lý `WM_SHOWWINDOW` với wParam TRUE để xóa cờ, bao phủ cả đường mở từ bản thứ hai.
- **M2 chế độ gắn kèm khi ẩn xuống khay**: khi ẩn cửa sổ chính (WM_CLOSE về khay, `--tray`), ẩn cửa sổ Zalo đang gắn; khi `show_main` thì `place_all` hiện lại. Không đổi chế độ nhúng.
- **M5 đường dẫn Zalo cũ** (`App::start`): khi `session::start` lỗi do không tìm thấy file, đặt `self.zalo = None` để lần sau tìm lại.
- **L4 phiên tự thoát** (`tick`): xóa `has_new`, `pending`, `flash_pending` của phiên đã chết.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 2 và 3 trong `plan.md`.

## Rủi ro

- `WM_APP_RELAYOUT` có thể lặp nếu `layout` luôn bị gọi lồng: chỉ post khi chưa có thông điệp đang chờ (cờ `Cell<bool>` ở thread_local).
- Ẩn Zalo ở chế độ gắn kèm làm watcher đọc cửa sổ ẩn: UI Automation vẫn đọc được cửa sổ ẩn của Chromium khi trình hiển thị chưa bị dừng; ghi nhận để kiểm thử tay mục 6.
