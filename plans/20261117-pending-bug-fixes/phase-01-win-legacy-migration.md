# Phase 1. Windows: bản cũ và chuyển dữ liệu

## Yêu cầu
- Khi MultiZ còn chạy (mutex `Local\SudoSoftware.MultiZ.SingleInstance` hoặc cửa sổ class `MultiZMain`), Betacom không chạm vào dữ liệu: báo người dùng thoát bản cũ rồi thoát.
- `data_root` ghi log khi chuyển dữ liệu thành công, thất bại, hoặc bỏ qua vì `Betacom` đã tồn tại cùng thư mục cũ.
- `migrate_autorun` giữ trạng thái tắt trong `StartupApproved\Run` của tên cũ: nếu bản cũ bị tắt thì chép giá trị đó sang tên `Betacom`.

## File
- `windows/src/main.rs`, `windows/src/util.rs`, `windows/src/actions.rs`.

## Kiểm thử
- Build, clippy, fmt. Thử tay: chạy MultiZ cũ rồi mở `betacom.exe`.
