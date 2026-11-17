# Betacom

Chạy nhiều tài khoản **Zalo PC** trên cùng một máy tính Windows. Mỗi tài khoản có một vùng dữ liệu riêng và hiện thành một tab ở thanh bên trái.

Phát triển bởi **Betacom** — https://github.com/betacomagency2026/BetaZalo

- Dùng chính **ứng dụng Zalo PC gốc**: không sửa Zalo, không chèn mã vào Zalo.
- Mỗi tài khoản một thư mục dữ liệu riêng, không lẫn phiên đăng nhập.
- Chấm đỏ "Có tin nhắn mới" trên tab và thông báo ở góc phải màn hình, ghi rõ tài khoản nào nhận tin.
- Thanh tài khoản thu gọn được, chỉ còn ảnh đại diện.
- Tự chọn ảnh đại diện cho từng tài khoản để dễ phân biệt.
- Chạy ẩn dưới khay hệ thống, có thể khởi động cùng Windows.
- Một file `.exe` khoảng 0,5 MB, không cần cài, không cần quyền Admin.
- **Miễn phí, mã nguồn mở, không quảng cáo, không kết nối Internet.** Xem [PRIVACY.md](PRIVACY.md).

> Betacom là phần mềm độc lập, không phải sản phẩm chính thức và không liên kết với Zalo / VNG. "Zalo" là nhãn hiệu của chủ sở hữu tương ứng.

## Cách dùng

1. Cài Zalo PC bản chính thức (https://zalo.me/pc) nếu máy chưa có.
2. Giải nén file `.zip`, mở `betacom.exe`. Lần đầu Windows có thể hiện "Windows protected your PC": bấm **More info → Run anyway**.
3. Bấm **Thêm tài khoản**, đặt tên, rồi quét mã QR bằng Zalo trên điện thoại.
4. Thêm các tài khoản khác tương tự. Bấm vào tab bên trái để chuyển tài khoản.

Mẹo:

- Chuột phải vào một tài khoản để đổi tên, đổi màu, **đổi ảnh đại diện**, tắt thông báo riêng, bật/tắt tự mở hoặc xóa.
- Nút **«** cạnh chữ Betacom để thu gọn thanh tài khoản, **»** để mở rộng lại.
- **Cài đặt**: thông báo, ẩn nội dung tin trong thông báo, chạy ẩn dưới khay, khởi động cùng Windows.
- Bấm **Tắt** để tắt một tài khoản; tài khoản đó giữ nguyên trạng thái tắt cho tới khi bạn bấm **Mở Zalo**.
- **Cài đặt → Kiểm tra phiên bản mới** mở trang https://github.com/betacomagency2026/BetaZalo/releases (Betacom không tự kết nối mạng để kiểm tra).
- Nếu Zalo hiện trắng/đen hoặc không gõ được chữ khi nhúng: **Cài đặt → Hiển thị: cửa sổ Zalo gắn kèm**.
- Bấm X thì Betacom chạy ẩn dưới khay để vẫn nhận thông báo. Muốn thoát hẳn: chuột phải biểu tượng Betacom ở khay → **Thoát Betacom**.

## Dữ liệu

```
%LOCALAPPDATA%\Betacom\
  config.json                     cài đặt + danh sách tài khoản
  log.txt                         nhật ký kỹ thuật (không chứa tin nhắn)
  profiles\<id>\                  dữ liệu Zalo của từng tài khoản
  profiles\<id>\avatar.png        ảnh đại diện bạn chọn (96×96)
```

Khi mở lần đầu, dữ liệu của bản cũ trong `%LOCALAPPDATA%\MultiZ`, `MultiZalo` hoặc `Zaloo` được chuyển sang `%LOCALAPPDATA%\Betacom`; mục khởi động cùng Windows tên `MultiZ` hoặc `Zaloo` được thay bằng `Betacom`. Thoát bản cũ trước khi mở bản mới.

## Cấu trúc mã nguồn

| File | Nội dung |
|---|---|
| `src/main.rs` | Trạng thái app, vòng lặp thông điệp, xử lý cửa sổ chính |
| `src/session.rs` | Chạy Zalo.exe với thư mục dữ liệu riêng (Job Object), tìm cửa sổ Zalo |
| `src/embed.rs` | Nhúng / gắn kèm cửa sổ Zalo vào Betacom |
| `src/ui.rs` | Vẽ giao diện (GDI+ / GDI) |
| `src/actions.rs` | Menu, hộp thoại, các thao tác của người dùng |
| `src/watcher.rs` | Phát hiện tin mới qua UI Automation (luồng riêng) |
| `src/tray.rs` | Biểu tượng khay hệ thống và thông báo |
| `src/avatar.rs` | Ảnh đại diện tự chọn |
| `src/dialog.rs`, `src/tooltip.rs`, `src/gfx.rs`, `src/config.rs`, `src/util.rs` | Phần phụ trợ |

## Build

Cần Rust (stable). Toolchain MSVC dùng `rc.exe` của Windows SDK (tự tìm, hoặc đặt biến `RC`); toolchain GNU dùng `windres` của mingw-w64 (biến `WINDRES`, mặc định `x86_64-w64-mingw32-windres`). Chạy trong thư mục `windows/`:

```
cargo build --release
```

File chạy nằm ở `windows\target\release\betacom.exe`. Biên dịch chéo từ Linux: target `x86_64-pc-windows-gnu` + mingw-w64.

## Giấy phép

MIT — Copyright (c) 2026 Betacom. Xem [LICENSE](../LICENSE).
