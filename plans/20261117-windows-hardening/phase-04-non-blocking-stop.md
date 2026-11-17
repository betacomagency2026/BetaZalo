# Phase 4: Tắt phiên không chặn UI

## Yêu cầu

- Bấm Tắt hoặc Khởi động lại khi Zalo đứng hình không làm MultiZ treo.
- Khởi động lại chỉ mở phiên mới khi phiên cũ đã thoát hẳn (profile hết bị khóa).

## Cách làm

- `App::stop`: `ShowWindowAsync(SW_HIDE)` thay cho `ShowWindow`; gọi `TerminateJobObject` rồi chuyển job vào `stopping: HashMap<String, Stopping { job, since, restart }>` thay vì chờ 3 giây.
- `tick`: với mỗi mục `stopping`, nếu `job_pids` rỗng hoặc quá 5 giây thì `CloseHandle`; nếu `restart` thì gọi `start`.
- `App::start`: bỏ qua khi id đang trong `stopping` và đánh dấu `restart = true`, tránh mở hai Zalo trên cùng profile.
- Trạng thái hiển thị: tài khoản đang trong `stopping` hiện "Đang tắt…".
- `exit_app`: giữ chờ đồng bộ có giới hạn (thoát chương trình, không còn UI cần phản hồi), dùng cùng hàm dừng.
- `session::stop` tách thành `terminate` (không chờ) và `wait_exit(timeout)`.

## Kiểm chứng

- Build, clippy, fmt.
- Kiểm thử tay mục 4 và 5 trong `plan.md`.

## Rủi ro

- Xóa tài khoản ngay sau khi tắt: `remove_dir_all` có thể gặp file còn khóa. `delete_account` gọi `wait_exit` có giới hạn trước khi xóa (người dùng đã xác nhận thao tác phá hủy, chờ ngắn chấp nhận được).
