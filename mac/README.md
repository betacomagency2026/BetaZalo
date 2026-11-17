# Betacom cho Mac

Phiên bản 0.3.4.

Chạy nhiều tài khoản **Zalo** trên cùng một máy Mac. Mỗi tài khoản có một thư mục dữ liệu riêng; Betacom nằm gọn trên thanh menu (góc trên bên phải màn hình) để mở và chuyển nhanh giữa các tài khoản.

Phát triển bởi **Betacom** — https://github.com/betacomagency2026/BetaZalo

- Dùng chính **ứng dụng Zalo gốc**: không sửa Zalo, không chèn mã vào Zalo.
- Nhẹ (khoảng 1 MB), không quảng cáo, **không kết nối Internet**, không thu thập dữ liệu.
- Chạy từ macOS 12 Monterey trở lên, máy Intel và chip Apple.
- Miễn phí, mã nguồn mở (MIT).

> Betacom không phải sản phẩm chính thức và không liên kết với Zalo / VNG.

## Cách dùng

1. Cài **Zalo cho Mac** bản tải từ https://zalo.me/pc (nên dùng bản này thay vì bản App Store).
2. Mở Betacom. Biểu tượng hai bong bóng chat xuất hiện trên thanh menu.
3. Bấm biểu tượng → **Thêm tài khoản…** → đặt tên → một cửa sổ Zalo mới mở ra → quét mã QR bằng Zalo trên điện thoại.
4. Thêm các tài khoản khác tương tự. Bấm tên tài khoản trong menu để chuyển tới Zalo của tài khoản đó (hoặc ⌘1, ⌘2… khi menu đang mở).

Trong menu:

- Chấm xanh trên ảnh đại diện = tài khoản đang chạy.
- **Quản lý tài khoản**: mở/tắt, đổi tên, đổi ảnh đại diện, xóa.
- **Tắt tất cả Zalo**, **Mở Betacom cùng máy Mac**, **Kiểm tra phiên bản mới…**

Lần đầu bấm chuyển tài khoản, macOS có thể hỏi *"Betacom muốn điều khiển Zalo"* — chọn **OK** để Betacom mở lại được cửa sổ Zalo khi bạn đã đóng nó.

## Build từ mã nguồn

Cần bộ công cụ dòng lệnh của Apple (miễn phí, không cần cài cả Xcode):

```bash
xcode-select --install     # chỉ cần làm một lần
bash build.sh
open build/Betacom.app
```

Kết quả: `build/Betacom.app` và `build/Betacom-mac.zip` (để chia sẻ cho người khác).

## Nâng cấp từ bản cũ (MultiZ, Zaloo)

Thoát bản cũ (và các Zalo do bản cũ mở) trước khi mở Betacom lần đầu. Betacom tự chuyển thư mục `~/Library/Application Support/MultiZ` hoặc `Zaloo` sang `Betacom` và chuyển mục "mở cùng máy Mac", các tài khoản vẫn giữ đăng nhập. macOS hỏi lại quyền điều khiển Zalo vì định danh ứng dụng đã đổi. Có thể xóa `MultiZ.app` hoặc `Zaloo.app` cũ trong Applications.

## Dữ liệu

```
~/Library/Application Support/Betacom/
  config.json              danh sách tài khoản
  log.txt                  nhật ký kỹ thuật (không chứa tin nhắn)
  profiles/<id>/           dữ liệu Zalo của từng tài khoản + avatar.png
```

Xóa tài khoản trong Betacom sẽ chuyển thư mục dữ liệu của tài khoản đó vào **Thùng rác**.

## Cách hoạt động

Betacom chạy Zalo với biến môi trường `CFFIXED_USER_HOME` trỏ vào `profiles/<id>`, nên Zalo lưu dữ liệu vào đó thay vì thư mục người dùng. Keychain vẫn dùng chung của máy (qua một liên kết) để Zalo lưu khóa bình thường.

| File | Nội dung |
|---|---|
| `Sources/main.swift` | Khởi động app (chỉ trên thanh menu, không có biểu tượng Dock) |
| `Sources/AppDelegate.swift` | Menu và các thao tác |
| `Sources/ZaloLauncher.swift` | Tìm Zalo, mở/tắt/chuyển phiên Zalo |
| `Sources/Store.swift` | Lưu danh sách tài khoản, nhật ký |
| `Sources/Avatar.swift` | Ảnh đại diện (chữ viết tắt hoặc ảnh tự chọn) |
| `Sources/LoginItem.swift` | Mở cùng máy Mac |
| `Sources/Prompt.swift` | Hộp thoại |

## Giấy phép

MIT — Copyright (c) 2026 Betacom. Xem [LICENSE](../LICENSE).
