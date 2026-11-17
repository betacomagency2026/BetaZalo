# Tiến độ

## Trạng thái

Plan `plans/20261117-windows-hardening`: xong mã cả 6 phase; build, clippy, fmt đạt; chưa kiểm thử tay với Zalo thật. Đã đổi tên sản phẩm thành Betacom.

Plan `plans/20261117-pending-bug-fixes`: xong mã 5 phase. Windows build, clippy, fmt đạt; macOS biên dịch đạt trên GitHub Actions (Swift 6.3.3), chưa chạy thử trên máy Mac.

## Nhật ký

### 2026-11-17

- Clone repo, đọc toàn bộ mã nguồn `windows/` và `mac/`.
- Viết `CLAUDE.md`: lệnh build, kiến trúc Windows và macOS.
- Dọn tài liệu trùng: một `LICENSE` ở gốc; README gốc thành trang tổng quan; README và PRIVACY chi tiết nằm trong từng thư mục nền tảng.
- Audit bản Windows (độ đúng, hiệu năng) và bản macOS; lập plan gia cố 6 phase; red-team rồi sửa plan.
- Phase 1: `build.rs` dùng `rc.exe` cho target MSVC (`windres` cho GNU); sửa 8 cảnh báo clippy.
- Phase 2: nạp icon khay một lần; chép ảnh đại diện vào bộ nhớ để nhả khóa file; config đọc lỗi thì chạy chế độ không lưu, sao lưu file hỏng theo thời điểm, lưu có `sync_all`; khóa log; xóa tài khoản kiểm tra id, thử lại và báo đường dẫn còn sót.
- Phase 3: hoãn bố cục khi window procedure bị gọi lồng (`WM_APP_RELAYOUT`); bỏ qua thu nhỏ; trao focus khi mở từ khay; xóa cờ khay qua `WM_SHOWWINDOW`; tìm lại Zalo.exe khi đường dẫn cũ mất; dừng tiến trình nếu không gán được vào Job Object; xóa trạng thái tin mới khi phiên thoát.
- Phase 4: tắt phiên không chờ trên luồng UI (`stopping`, thời hạn 10 giây, trạng thái "Đang tắt…"); khởi động lại chỉ mở sau khi phiên cũ thoát; bỏ qua cửa sổ đứng hình.
- Phase 5: watcher tạo truy vấn UI Automation một lần; bỏ tiêu đề cửa sổ Zalo khỏi log.
- Phase 6: cắt chuỗi khay không tách cặp surrogate; Enter trên nút Hủy đóng hộp thoại như Hủy.
- Kiểm tra: `cargo build --release`, `cargo clippy --release` không cảnh báo, `cargo fmt --check` đạt. Bản macOS chưa kiểm chứng (máy không có Swift).
- Đổi tên MultiZ thành Betacom (mã, tài nguyên exe, bundle macOS, tài liệu, LICENSE); bỏ nút và mục menu "Mời cà phê"; chuyển dữ liệu, mục khởi động cùng Windows và LaunchAgent của bản MultiZ sang tên mới. Build, clippy, fmt bản Windows đạt; bản macOS chưa biên dịch thử.
- Rà soát sau đổi tên; lập plan `20261117-pending-bug-fixes`.
- Windows phase 1: từ chối chạy khi bản MultiZ cũ đang chạy (mutex, class cửa sổ); ghi log kết quả chuyển thư mục dữ liệu; chuyển trạng thái `StartupApproved\Run` sang tên mới.
- Windows phase 2: xóa tài khoản khỏi config trước, dọn profile ở luồng phụ qua thư mục `.trash-*`, dọn sót khi khởi động; báo chế độ không lưu; gom lưu config khi chọn tài khoản (2 giây).
- Windows phase 3: không báo trùng tin khi nhấp nháy taskbar; cộng dồn delta cuộn chuột; SetParent lỗi thì chuyển cửa sổ đó sang gắn kèm; thông báo lỗi CreateProcessW dễ hiểu; sửa câu `SHA256SUMS.txt` trong PRIVACY; gỡ comment.
- macOS phase 4: file pid kèm thời điểm khởi động tiến trình; giữ `Process` và `terminationHandler`; chỉ gửi tín hiệu cho tiến trình đã xác minh; xóa tài khoản chỉ dọn profile sau khi Zalo thoát; "Tắt Zalo và thoát" chờ qua `applicationShouldTerminate`.
- macOS phase 5: không ghi LaunchAgent khi chạy từ App Translocation, thêm `AbandonProcessGroup`; mở lại tài khoản cách 1,5 giây; tạo lại symlink Keychain sai; chặn chạy hai bản; sao lưu config hỏng theo thời điểm, chế độ không lưu, giữ `log.old.txt`; báo thiếu quyền Automation; `build.sh` không nuốt lỗi arm64 và lỗi ký; gỡ comment.
- Rà soát lần hai (hai reviewer độc lập), sửa 16 lỗi:
  - Windows: tài khoản không nhúng được xử lý theo chế độ gắn kèm (di chuyển, focus, Zalo tự hiện cửa sổ); nút khởi động cùng Windows đọc và xóa trạng thái tắt trong Task Manager; hoãn hộp báo lỗi xóa dữ liệu khi hộp thoại tài khoản đang mở; dọn `.trash-*` không báo lỗi giả khi thư mục đã mất; lưu lựa chọn khi đăng xuất hoặc tắt máy; chỉ chặn khi MultiZ chạy nếu còn dữ liệu cần chuyển.
  - macOS: nhận lại phiên chỉ dựa vào PID và thời điểm khởi động (không phụ thuộc đường dẫn Zalo.app); kiểm tra id tài khoản, chặn dọn profile với id sai hoặc đang dùng; ghi LaunchAgent mới trước khi gỡ cái cũ, chỉ ghi khi nội dung đổi, báo khi tắt thất bại; không ghi lại config mỗi lần khởi động; gửi SIGTERM cho phiên đã xác minh khi không có `NSRunningApplication`; buộc tắt sau 8 giây, bỏ chờ sau 10 giây; chặn crash khi màu âm; bản mở lại trong lúc bản cũ đang thoát thì chờ bản cũ thoát hẳn.
- Thêm workflow `build-mac.yml`: build bản macOS trên máy `macos-latest` của GitHub Actions (`macos-14` không còn được hỗ trợ). Lần chạy đầu biên dịch đạt, chỉ còn cảnh báo `fullPath(forApplication:)` đã lỗi thời; `Betacom-mac.zip` (universal, ký ad-hoc) lấy từ artifact.

## Việc tiếp theo

- Chạy thử bản macOS từ artifact `Betacom-mac.zip` trên máy Mac: mở/tắt, thoát Betacom rồi mở lại (nhận lại phiên), xóa tài khoản đang chạy, "Tắt Zalo và thoát", mở hai bản, chạy từ thư mục Downloads (App Translocation), từ chối quyền Automation, đổi vị trí Zalo.app rồi mở lại Betacom, thoát rồi mở lại ngay trong lúc đang tắt Zalo.
- Kiểm thử tay bản Windows sau plan sửa lỗi: mở khi MultiZ cũ đang chạy, xóa tài khoản đang chạy, thông báo khi Zalo nhấp nháy taskbar, cuộn bằng touchpad, bật lại khởi động cùng Windows sau khi tắt trong Task Manager.
- Chưa rõ mutex và class cửa sổ của bản Zaloo, MultiZalo cũ (không có trong lịch sử repo) nên chưa chặn khi chúng đang chạy.

- Kiểm thử tay việc đổi tên: thoát bản MultiZ, mở `betacom.exe`; dữ liệu `%LOCALAPPDATA%\MultiZ` chuyển sang `Betacom`, tài khoản giữ đăng nhập, thanh bên không còn nút cà phê.
- Cung cấp website chính thức của Betacom nếu có, thay liên kết GitHub ở "Trang chủ Betacom".
- Chạy 7 mục kiểm thử tay trong `plans/20261117-windows-hardening/plan.md` trên máy có Zalo.
- Các mục hoãn cần thử với Zalo thật: M2 (ẩn cửa sổ khi xuống khay, chế độ gắn kèm), bỏ `BREAKAWAY_OK`, đóng Zalo êm khi tắt máy (M7, W9), Folder Redirection (L10), chọn nhầm cửa sổ phụ (M9), menu khay mở hai lần (L1).
- Timeout UI Automation: cần `CUIAutomation8` và số liệu thời gian đọc p50/p95.
- macOS M7 phần Keychain dùng chung: thử trên máy thật xem Zalo có ghi lẫn Keychain giữa các tài khoản không.
- macOS: thay `NSWorkspace.fullPath(forApplication:)` đã lỗi thời bằng `urlForApplication(withBundleIdentifier:)`.
- Thống nhất hành vi xóa tài khoản giữa hai bản (Windows xóa hẳn, macOS chuyển vào Thùng rác).
- Đánh giá độ bền của `watcher.rs` trước thay đổi bố cục giao diện Zalo.
- Windows: Betacom thoát trong lúc chờ Zalo thoát khi xóa tài khoản thì thư mục profile bị bỏ lại (không tự xóa profile mồ côi để tránh mất dữ liệu khi config bị đặt lại).
- `UPDATE_URL` trỏ tới trang releases còn trống cho tới khi có quy trình phát hành.
- Kiểm chứng khi thử tay: Zalo có lưu đường dẫn tuyệt đối (thư mục tải, tệp cục bộ) trong profile không; nếu có, tệp cũ mất sau khi đổi `MultiZ` thành `Betacom`.
- Hướng phát triển: CI GitHub Actions (build, clippy, fmt) và phát hành `betacom.exe` kèm `SHA256SUMS.txt`; phím tắt Ctrl+1..9, nhớ vị trí cửa sổ, toast kích hoạt đúng tài khoản (W7, W10).
