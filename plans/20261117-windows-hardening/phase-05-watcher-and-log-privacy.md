# Phase 5: Watcher và riêng tư log

## Yêu cầu và cách làm

- **W4 một phần** (`watcher.rs`): tạo `condition` và `cache request` một lần khi khởi động luồng, truyền vào `read_lines`. Giữ nguyên thuật toán, chu kỳ và việc đọc tài khoản đang xem.
- **L8** (`embed.rs`, `main.rs`): bỏ tiêu đề cửa sổ Zalo khỏi log (có thể là tên người liên hệ), chỉ ghi class và kích thước.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 6 trong `plan.md`; log không còn tiêu đề cửa sổ.

## Hoãn

- Timeout UI Automation: cần tạo `CUIAutomation8` và có số liệu thời gian đọc p50/p95 trước khi chọn ngưỡng.
