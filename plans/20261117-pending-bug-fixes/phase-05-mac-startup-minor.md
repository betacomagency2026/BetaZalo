# Phase 5. macOS: khởi động và lỗi nhỏ

## Yêu cầu
- M4: không ghi LaunchAgent khi đường dẫn chứa `/AppTranslocation/`, báo người dùng chuyển app vào Applications; thêm `AbandonProcessGroup`.
- M6: mở lại tài khoản lần lượt cách 1,5 giây.
- M7: symlink Keychain gãy hoặc trỏ sai thì tạo lại; ghi log khi lỗi.
- M8: đã có bản Betacom khác chạy thì kích hoạt bản đó và thoát.
- M9: sao lưu config hỏng theo thời điểm; log giữ `log.old.txt`, ghi bằng API không ném ngoại lệ ObjC, dùng lại `DateFormatter`; báo khi thiếu quyền Automation; `build.sh` không nuốt lỗi biên dịch arm64 và lỗi ký.

## File
- `mac/Sources/LoginItem.swift`, `AppDelegate.swift`, `ZaloLauncher.swift`, `Store.swift`, `main.swift`, `mac/build.sh`.
