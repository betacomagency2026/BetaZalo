# Sửa lỗi tồn đọng Windows và macOS

Trạng thái: đang làm · Bắt đầu: 2026-11-17

Mục tiêu: sửa các lỗi đã xác nhận còn mở ở cả hai bản trước khi phát triển tính năng mới. Không đổi hành vi người dùng thấy được ngoài chỗ sửa lỗi.

Nguồn: rà soát sau đổi tên (`docs/tien-do.md`, mục "Việc tiếp theo"); [audit hiệu năng và macOS](../20261117-windows-hardening/reports/audit-performance-mac.md) (W8, W10, M1–M9).

## Phase

| # | Phase | Mục | Trạng thái |
|---|---|---|---|
| 1 | [Windows: bản cũ và chuyển dữ liệu](phase-01-win-legacy-migration.md) | mutex MultiZ, log chuyển dữ liệu, `StartupApproved` | xong |
| 2 | [Windows: xóa tài khoản và lưu config](phase-02-win-delete-and-config.md) | xóa ở luồng phụ, `read_only`, lưu khi chọn | chờ |
| 3 | [Windows: thông báo và chi tiết giao diện](phase-03-win-notify-ui.md) | W8, cuộn, SetParent, mã lỗi, PRIVACY, comment | chờ |
| 4 | [macOS: vòng đời tiến trình](phase-04-mac-process-lifecycle.md) | M1, M2, M3, M5 | chờ |
| 5 | [macOS: khởi động và lỗi nhỏ](phase-05-mac-startup-minor.md) | M4, M6, M7 (sửa link), M8, M9 | chờ |

## Tiêu chí hoàn thành

- Windows: `cargo build --release`, `cargo clippy --release` không cảnh báo, `cargo fmt --check` đạt.
- macOS: mã Swift viết theo API có tài liệu; chưa biên dịch được trên máy này, cần `bash build.sh` trên macOS.
- Mỗi phase một commit.

## Ngoài phạm vi

- Các mục cần Zalo thật đã hoãn: M2/M3 của audit Windows, `BREAKAWAY_OK`, W9, L1, L10, timeout UIA.
- W7 (toast kích hoạt đúng tài khoản), W6, phím tắt, nhớ vị trí cửa sổ, CI và phát hành: thuộc giai đoạn phát triển.
- M7 phần Keychain dùng chung: cần thử trên máy thật.
