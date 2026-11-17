# Phase 4: Tắt phiên không chặn UI

## Yêu cầu

- Bấm chọn, Tắt hoặc Khởi động lại khi Zalo đứng hình không làm MultiZ treo.
- Không bao giờ có hai Zalo trên cùng một profile.
- Tài khoản đã xóa không bao giờ được mở lại.

## Cách làm

- `session.rs`: tách `stop` thành `terminate(&Session)` (không chờ) và `wait_exit(job, timeout)`.
- `App::stop`: `ShowWindowAsync(SW_HIDE)`; `terminate`; chuyển job vào `stopping: HashMap<String, Stopping { job, since, restart }>`.
- `tick`: mỗi mục `stopping`:
  - `job_pids` rỗng: `CloseHandle`; nếu `restart` và tài khoản còn trong config thì `start`.
  - Quá 10 giây chưa rỗng: `CloseHandle`, không mở lại, ghi `failed` = "Phiên cũ chưa thoát hẳn, hãy thử lại".
- `App::start`: bỏ qua nếu tài khoản không còn trong config; nếu id đang trong `stopping` thì chỉ đặt `restart = true`.
- Khởi động lại: `stop` với `restart = true`.
- `delete_account`: gỡ mục `stopping` của id, `wait_exit` có giới hạn 3 giây rồi mới xóa thư mục (thao tác phá hủy người dùng đã xác nhận).
- Cửa sổ đứng hình (`embed.rs`): `place`, `park`, `activate_child` bỏ qua khi `IsHungAppWindow(hwnd)`; nhánh `tick` gọi `ShowWindow`/`SetWindowPos` cũng kiểm tra.
- Trạng thái hiển thị "Đang tắt…" cho id trong `stopping`; thêm `stopping` vào chuỗi chữ ký vẽ lại.
- `exit_app`: giữ nguyên hành vi (đóng job, `KILL_ON_JOB_CLOSE` kết thúc tiến trình).

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 4 và 5 trong `plan.md`.
