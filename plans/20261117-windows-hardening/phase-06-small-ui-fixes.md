# Phase 6: Sửa nhỏ giao diện

## Yêu cầu và cách làm

- **L3 cắt chuỗi giữa cặp surrogate** (`tray.rs::copy`): nếu ký tự cuối được giữ là high surrogate thì bỏ nó.
- **L9 Enter trên nút Hủy** (`dialog.rs:317`): xử lý `IDOK` theo nút đang có focus; Enter trên Hủy đóng hộp thoại như Hủy.

## Kiểm chứng

- Build, clippy, fmt.
- Đặt tên tài khoản dài có emoji, bật thông báo: không còn ký tự lỗi ở cuối.
