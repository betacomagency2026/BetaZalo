# Phase 3. Windows: thông báo và chi tiết giao diện

## Yêu cầu
- W8: tin do watcher bắt được trong `NOTIFY_COOLDOWN` sau thông báo chung từ nhấp nháy taskbar không được đếm thành tin thứ hai.
- Cuộn chuột cộng dồn delta, mỗi 120 đơn vị một dòng.
- `SetParent` lỗi: khôi phục style gốc, không để cửa sổ Zalo mất khung.
- `CreateProcessW` lỗi 2, 5, 740: câu báo dễ hiểu.
- `windows/PRIVACY.md`: bỏ câu khẳng định mọi bản phát hành kèm `SHA256SUMS.txt`.
- Bỏ comment kế thừa trong `windows/src`.

## File
- `windows/src/*.rs`, `windows/PRIVACY.md`.
