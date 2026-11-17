# Gia cố bản Windows

Trạng thái: đang thực thi · Bắt đầu: 2026-11-17

Mục tiêu: sửa các lỗi đã xác nhận qua audit (rò tài nguyên, mất dữ liệu, gọi lồng làm hỏng giao diện, treo UI), giảm chi phí theo dõi tin mới; giữ nguyên các điểm mạnh: một file exe nhỏ, không kết nối mạng, không sửa Zalo, dữ liệu tách theo tài khoản.

Nguồn: [audit độ đúng](reports/audit-windows-correctness.md), [audit hiệu năng và macOS](reports/audit-performance-mac.md), [red-team](reports/red-team-plan-review.md).

## Phase

| # | Phase | Mục audit | Trạng thái |
|---|---|---|---|
| 1 | [Build và lint](phase-01-build-and-lint.md) | build.rs, clippy | đang làm |
| 2 | [Rò tài nguyên và an toàn dữ liệu](phase-02-resources-and-data-safety.md) | H1, H2/W3, M1, M8, L6, L7 | chờ |
| 3 | [Gọi lồng và trạng thái cửa sổ](phase-03-reentrancy-and-window-state.md) | H3, L2, M2, M4, M5, L4 | chờ |
| 4 | [Tắt phiên không chặn UI](phase-04-non-blocking-stop.md) | W2, W5, L5 | chờ |
| 5 | [Watcher và riêng tư log](phase-05-watcher-and-log-privacy.md) | W4, L8 | chờ |
| 6 | [Sửa nhỏ giao diện](phase-06-small-ui-fixes.md) | L3, L9 | chờ |

Thứ tự: 1 → 2 → 3 → 4 → 5 → 6. Phase 4 phụ thuộc phase 3 (cùng sửa `stop`, `tick`).

## Tiêu chí hoàn thành

- `cargo build --release`, `cargo clippy --release` không cảnh báo, `cargo fmt --check` đạt.
- Mỗi phase có commit riêng; hành vi người dùng thấy được giữ nguyên trừ chỗ sửa lỗi.
- Danh sách kiểm thử tay cuối plan được người dùng chạy trên máy có Zalo.

## Ngoài phạm vi

- Bản macOS: cần máy macOS để biên dịch; lập plan riêng.
- Cần thử với Zalo thật trước khi quyết: bỏ `JOB_OBJECT_LIMIT_BREAKAWAY_OK` (M6), đóng Zalo êm khi tắt máy (M7, W9), Folder Redirection (L10), chọn nhầm cửa sổ phụ (M3, M9), menu khay mở hai lần (L1).
- Bấm thông báo cũ trong Action Center mở sai tài khoản (W7): API khay không trả định danh thông báo.
- Tính năng mới (phím tắt, nhớ vị trí cửa sổ) và giảm chi phí kéo cửa sổ (W6).

## Kiểm thử tay

1. Thêm 2 tài khoản, đặt ảnh đại diện, đổi ảnh lần hai, "Dùng lại chữ viết tắt", rồi xóa tài khoản: thư mục profile phải biến mất.
2. Thu nhỏ MultiZ, bấm biểu tượng khay: vùng Zalo hiện đủ, gõ phím được ngay.
3. Chế độ gắn kèm: đóng xuống khay thì cửa sổ Zalo ẩn theo; mở lại thì hiện đúng chỗ.
4. Làm Zalo đứng hình (tạm dừng tiến trình trong Resource Monitor), bấm Tắt: MultiZ không treo.
5. "Khởi động lại" một tài khoản: phiên mới mở sau khi phiên cũ thoát hẳn.
6. Gửi tin tới tài khoản không chọn: vẫn có chấm đỏ và thông báo; để chạy vài giờ, số GDI/USER object (Task Manager) không tăng dần.
