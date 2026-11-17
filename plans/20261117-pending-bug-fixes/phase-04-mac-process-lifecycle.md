# Phase 4. macOS: vòng đời tiến trình

## Yêu cầu
- M1: file pid ghi kèm thời điểm khởi động tiến trình (`proc_pidinfo`, `PROC_PIDTBSDINFO`); `restore` chỉ nhận khi khớp, sai thì xóa file pid.
- M2: giữ `Process`, dùng `terminationHandler`; bỏ `kill(pid, SIGTERM)` cho tiến trình không xác minh được.
- M3: xóa tài khoản chỉ chuyển profile vào Thùng rác sau khi tiến trình đã thoát (có giới hạn thời gian, sau bước buộc tắt).
- M5: "Tắt Zalo và thoát" chờ mọi phiên thoát (tối đa khoảng 7 giây) qua `applicationShouldTerminate` trả `.terminateLater`.

## File
- `mac/Sources/ZaloLauncher.swift`, `mac/Sources/AppDelegate.swift`.

## Kiểm thử
- Cần macOS: `bash build.sh`, mục thử trong `mac/HUONG-DAN-CAI-DAT.md`.
