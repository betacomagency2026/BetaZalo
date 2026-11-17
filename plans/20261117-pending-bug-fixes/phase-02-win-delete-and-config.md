# Phase 2. Windows: xóa tài khoản và lưu config

## Yêu cầu
- Xóa tài khoản: bỏ khỏi config và lưu trước; đổi tên thư mục profile thành `profiles\.trash-<id>-<thời điểm>`; chờ phiên thoát và xóa ở luồng phụ; khởi động thì dọn các `.trash-*` còn sót.
- Chế độ `read_only`: báo một lần kèm đường dẫn `config.json`; không xóa thư mục profile.
- `select` không ghi đĩa mỗi lần bấm: đánh dấu cần lưu, `tick` lưu sau 2 giây, thoát ứng dụng thì lưu ngay.

## File
- `windows/src/actions.rs`, `windows/src/main.rs`, `windows/src/util.rs`.

## Rủi ro
- Đổi tên thư mục lỗi khi Zalo còn giữ file: chờ phiên thoát ở luồng phụ rồi đổi tên; vẫn lỗi thì xóa tại chỗ và ghi log.
