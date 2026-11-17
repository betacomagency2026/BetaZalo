# Hướng dẫn chép sang Mac và chạy Betacom

Làm một lần, mất khoảng 10–15 phút (phần lớn là chờ cài công cụ của Apple).

## Bước 1 — Đưa file sang Mac

Chọn một cách:

- **Đơn giản nhất:** trên Mac, mở trình duyệt, vào lại cuộc trò chuyện này và tải file `betacom-mac-source.zip` trực tiếp.
- Hoặc chép bằng USB, AirDrop, Google Drive… từ máy Windows sang Mac.

Bấm đúp file `.zip` trên Mac để giải nén. Bạn sẽ có thư mục `betacom-mac` trong **Downloads** (Tải về).

## Bước 2 — Cài công cụ build của Apple (chỉ một lần)

1. Mở **Terminal**: bấm `⌘ + Space`, gõ `Terminal`, Enter.
2. Dán lệnh sau rồi Enter:

   ```
   xcode-select --install
   ```

3. Một cửa sổ hiện ra → bấm **Install** (Cài đặt) → **Agree** → đợi cài xong (5–10 phút).

   Nếu Terminal báo *"command line tools are already installed"* thì máy đã có sẵn, sang bước 3.

## Bước 3 — Build Betacom

Trong Terminal, dán lần lượt:

```
cd ~/Downloads/betacom-mac
bash build.sh
```

Đợi khoảng 1 phút. Khi thấy dòng **"Xong!"** là thành công.

## Bước 4 — Cài và mở

```
cp -R build/Betacom.app /Applications/
open /Applications/Betacom.app
```

Biểu tượng **hai bong bóng chat** sẽ hiện trên thanh menu ở góc trên bên phải màn hình. Bấm vào đó để thêm tài khoản.

Vì bạn tự build trên máy mình nên macOS sẽ không chặn. Nếu sau này chia sẻ file `build/Betacom-mac.zip` cho người khác, lần đầu họ cần **chuột phải vào Betacom → Open (Mở)** để macOS cho chạy.

## Nếu gặp lỗi

- Chụp màn hình Terminal (nếu lỗi lúc build), hoặc
- Mở menu Betacom → **Mở thư mục dữ liệu** → gửi file `log.txt`

rồi gửi cho người phát triển.

## Những điều cần thử giúp

1. Thêm 2 tài khoản, đăng nhập 2 Zalo khác nhau → cả hai cùng chạy, không đăng xuất lẫn nhau.
2. Thoát Betacom rồi mở lại → hai tài khoản vẫn còn đăng nhập.
3. Đóng cửa sổ một Zalo (nút đỏ), rồi bấm tên tài khoản đó trong menu Betacom → cửa sổ hiện lại.
4. Gửi tin nhắn tới từng tài khoản → thông báo hiện bình thường.
