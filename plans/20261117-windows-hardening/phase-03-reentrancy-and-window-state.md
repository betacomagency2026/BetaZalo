# Phase 3: Gọi lồng và trạng thái cửa sổ

## Yêu cầu và cách làm

- **H3 bố cục bị bỏ** (`main.rs` WM_SIZE): bỏ qua `SIZE_MINIMIZED`; khi `with_app` trả `None` thì post `WM_APP_RELAYOUT` (chỉ một thông điệp chờ, cờ `Cell<bool>` thread_local); thông điệp này gọi `layout`. Cùng mẫu cho WM_DPICHANGED (dựng lại font) và WM_MOVE/WM_WINDOWPOSCHANGED.
- **L2 focus khi mở từ khay** (`show_main`): post `WM_APP_FOCUS` sau khi hiện cửa sổ.
- **M4 cờ hidden_to_tray**: xử lý `WM_SHOWWINDOW` wParam TRUE để xóa cờ; bao phủ cả đường mở từ bản thứ hai.
- **M3 cửa sổ phụ bị nhận làm cửa sổ chính** (`tick`, nhánh `is_sel`): thêm điều kiện `IsIconic(main) == 0` trước khi coi cửa sổ Zalo là bị ẩn.
- **M5 đường dẫn Zalo cũ** (`App::start`): nếu `self.zalo` không còn là file thì tìm lại trước khi mở.
- **M6 phần đầu** (`session::start`): `AssignProcessToJobObject` lỗi thì `TerminateProcess`, đóng handle, trả lỗi; không để Zalo chạy ngoài tầm quản lý (hiện tại phiên đó cũng bị coi là đã thoát vì job rỗng).
- **L4 phiên tự thoát** (`tick`) và `stop`: xóa `has_new`, `pending`, `flash_pending` của phiên.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 2 và 3 trong `plan.md`.

## Rủi ro

- `WM_APP_RELAYOUT` chỉ post khi chưa có thông điệp chờ, tránh vòng lặp.
