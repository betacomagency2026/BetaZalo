# Phase 5: Watcher và riêng tư log

## Yêu cầu và cách làm

- **W4 chi phí watcher** (`watcher.rs`), giữ nguyên thuật toán so tập dòng:
  - Tạo `condition` và `cache request` một lần khi khởi động luồng, không tạo lại mỗi vòng.
  - Không đọc cây UIA của tài khoản đang xem (`active`); chỉ đặt `seen_once = false` để lần đọc kế tiếp làm mốc, tránh báo tin cũ khi người dùng chuyển đi.
  - Đặt timeout kết nối và giao dịch qua `IUIAutomation2` (ví dụ 2 giây) để một Zalo đứng hình không chặn cả luồng.
- **L8 log tiêu đề cửa sổ** (`embed.rs:146`, `main.rs:519`): bỏ tiêu đề khỏi log, chỉ ghi class và kích thước.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 6 trong `plan.md`; log không còn tên hội thoại.

## Rủi ro

- Bỏ đọc tài khoản đang xem làm mất trạng thái `prev`; cách đặt lại `seen_once` giữ hành vi hiện tại (khi `active` thì mã cũ cũng chỉ cập nhật mốc).
- `IUIAutomation2` có từ Windows 8; Windows 7 không còn là mục tiêu.
