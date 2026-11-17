# Quyền riêng tư

Zaloo là phần mềm mã nguồn mở (MIT); mọi điều dưới đây đều kiểm tra được trong mã nguồn.

## Zaloo không kết nối Internet

Zaloo không có bất kỳ đoạn mã nào gửi hay nhận dữ liệu qua mạng: không quảng cáo, không thống kê sử dụng, không kiểm tra cập nhật. File `Zaloo.exe` không dùng thư viện mạng của Windows (WinHTTP, WinINet, Winsock).

Các phiên Zalo bên trong vẫn kết nối máy chủ của Zalo như khi bạn dùng Zalo bình thường; Zaloo không can thiệp vào kết nối đó.

Zaloo không tự kiểm tra cập nhật. Các link trong app (Sudo Software, Mời cà phê, Kiểm tra phiên bản mới, Tải Zalo PC) chỉ mở trình duyệt khi bạn bấm vào.

## Zaloo không làm gì với Zalo

- Không sửa file của Zalo, không chèn mã (DLL injection) vào Zalo.
- Không đọc cơ sở dữ liệu tin nhắn của Zalo.
- Không cần quyền Administrator.

## Thông báo tin nhắn mới

Để hiện chấm đỏ và thông báo, Zaloo đọc chữ đang hiển thị trong danh sách hội thoại của cửa sổ Zalo qua **UI Automation** — chuẩn hỗ trợ tiếp cận của Windows, giống trình đọc màn hình. Việc này diễn ra hoàn toàn trên máy bạn; nội dung không được lưu lại hay gửi đi.

## Dữ liệu trên máy

Tất cả nằm trong `%LOCALAPPDATA%\Zaloo\`:

- `config.json` — tên tài khoản do bạn đặt, màu và các cài đặt.
- `profiles\<id>\` — dữ liệu Zalo của từng tài khoản (do Zalo tạo ra) và ảnh đại diện bạn chọn.
- `log.txt` — nhật ký kỹ thuật (mở app, mở/tắt phiên Zalo, lỗi), không chứa nội dung tin nhắn.

Xóa một tài khoản trong Zaloo sẽ xóa thư mục dữ liệu của tài khoản đó trên máy.

## Kiểm tra file tải về

Mỗi bản phát hành kèm `SHA256SUMS.txt`. Kiểm tra trong PowerShell:

```
Get-FileHash .\Zaloo.exe -Algorithm SHA256
```
