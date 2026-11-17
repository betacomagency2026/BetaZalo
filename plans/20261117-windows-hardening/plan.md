# Gia cố bản Windows

Trạng thái: xong phần mã, chờ kiểm thử tay trên máy có Zalo · Bắt đầu: 2026-11-17

Mục tiêu: sửa các lỗi đã xác nhận qua audit (rò tài nguyên, mất dữ liệu, gọi lồng làm hỏng giao diện, treo UI), giảm chi phí theo dõi tin mới; giữ nguyên các điểm mạnh: một file exe nhỏ, không kết nối mạng, không sửa Zalo, dữ liệu tách theo tài khoản.

Nguồn: [audit độ đúng](reports/audit-windows-correctness.md), [audit hiệu năng và macOS](reports/audit-performance-mac.md), [red-team](reports/red-team-plan-review.md). Plan đã sửa theo red-team (B1, M-1 đến M-9 và các minor liên quan).

## Phase

| # | Phase | Mục audit | Trạng thái |
|---|---|---|---|
| 1 | [Build và lint](phase-01-build-and-lint.md) | build.rs, clippy | xong (87bad24) |
| 2 | [Rò tài nguyên và an toàn dữ liệu](phase-02-resources-and-data-safety.md) | H1, H2/W3, M1, M8, L6, L7 | xong (f1bc571) |
| 3 | [Gọi lồng và trạng thái cửa sổ](phase-03-reentrancy-and-window-state.md) | H3, L2, M3, M4, M5, M6 (một phần), L4 | xong (4806b46) |
| 4 | [Tắt phiên không chặn UI](phase-04-non-blocking-stop.md) | W2, W5, L5 | xong (72c66a4) |
| 5 | [Watcher và riêng tư log](phase-05-watcher-and-log-privacy.md) | W4 (một phần), L8 | xong (9c2d4f7) |
| 6 | [Sửa nhỏ giao diện](phase-06-small-ui-fixes.md) | L3, L9 | xong (f0eaea3) |

Thứ tự: 1 → 2 → 3 → 4 → 5 → 6. Phase 4 phụ thuộc phase 2 (thứ tự xóa tài khoản) và phase 3 (`stop`, `tick`).

## Tiêu chí hoàn thành

- `cargo build --release`, `cargo clippy --release` không cảnh báo, `cargo fmt --check` đạt.
- Mỗi phase một commit; hành vi người dùng thấy được giữ nguyên trừ chỗ sửa lỗi.
- Danh sách kiểm thử tay dưới đây được chạy trên máy có Zalo.

## Ngoài phạm vi

- Bản macOS: cần máy macOS để biên dịch; lập plan riêng.
- Cần thử với Zalo thật trước khi quyết:
  - Ẩn cửa sổ Zalo khi MultiZ xuống khay ở chế độ gắn kèm (M2): chưa có bằng chứng UI Automation còn đọc được cửa sổ ẩn.
  - Bỏ `JOB_OBJECT_LIMIT_BREAKAWAY_OK` (phần sau của M6).
  - Đóng Zalo êm khi tắt máy (M7, W9); Folder Redirection (L10).
  - Chọn nhầm cửa sổ phụ ở chế độ nhúng (M9); menu khay mở hai lần (L1).
- Timeout UI Automation (cần `CUIAutomation8` và số liệu p50/p95) và bỏ đọc tài khoản đang xem: red-team chỉ ra rủi ro mất tin.
- Bấm thông báo cũ trong Action Center mở sai tài khoản (W7): API khay không trả định danh thông báo.
- Tính năng mới (phím tắt, nhớ vị trí cửa sổ), giảm chi phí kéo cửa sổ (W6).

## Kiểm thử tay

1. Thêm 2 tài khoản, đặt ảnh đại diện, đổi ảnh lần hai, "Dùng lại chữ viết tắt", rồi xóa tài khoản: thư mục profile biến mất.
2. Thu nhỏ MultiZ, bấm biểu tượng khay: vùng Zalo hiện đủ, gõ phím được ngay.
3. Mở MultiZ lần hai khi bản đầu đang ở khay: cửa sổ hiện lại; tin tới tài khoản đang xem không bị báo.
4. Làm Zalo đứng hình (Resource Monitor → Suspend process): bấm chọn tài khoản đó rồi bấm Tắt, MultiZ không treo; trạng thái "Đang tắt…" rồi "Đã tắt".
5. "Khởi động lại": phiên mới chỉ mở sau khi phiên cũ thoát; bấm "Khởi động lại" rồi xóa ngay tài khoản: không có Zalo nào mở lại.
6. Gửi tin tới tài khoản không chọn: vẫn có chấm đỏ và thông báo; chạy vài giờ, số GDI/USER object trong Task Manager không tăng dần.
7. Khóa `config.json` (mở bằng chương trình giữ khóa ghi) rồi mở MultiZ: danh sách tài khoản không bị ghi đè.
