# Audit độ đúng và độ bền: bản Windows

Phạm vi: `windows/src/*.rs` (4139 dòng), đọc mã, không chạy. Bỏ qua các mục đã biết (heuristic watcher, thiếu test, cảnh báo clippy, build.rs).
Quy ước: **C** = confirmed bằng đọc mã (hoặc hành vi API có tài liệu); **P** = plausible, có kịch bản nhưng phụ thuộc hành vi Zalo/Windows.

## HIGH

### H1. Rò HICON mỗi lần gửi thông báo (C)
- `tray.rs:72-74` gọi `LoadImageW` không có `LR_SHARED` cho `hBalloonIcon` ở mỗi `notify`, và không gọi `DestroyIcon`. `tray.rs:32-39` cũng vậy mỗi lần `add` (khởi động, mỗi lần `TaskbarCreated`).
- Kịch bản: 5 tài khoản bán hàng, mỗi tài khoản báo tối đa 1 lần/8 giây (`NOTIFY_COOLDOWN`) → hàng nghìn HICON/giờ. Hạn mức USER object mặc định 10.000/tiến trình; khi chạm hạn, `CreateWindowExW`, `LoadImageW`, `TrackPopupMenu`, hộp thoại đều lỗi, MultiZ treo giao diện trong khi vẫn giữ mọi phiên Zalo.
- Sửa: nạp icon một lần (lưu trong `App`, hoặc dùng `LR_SHARED`), hoặc `DestroyIcon` ngay sau `Shell_NotifyIconW` (shell sao chép icon).

### H2. GDI+ giữ khóa `avatar.png`, nên xóa tài khoản không xóa hết dữ liệu và đổi ảnh lần hai bị lỗi (C)
- `gfx.rs:183`: `GdipCreateBitmapFromFile` giữ file mở, không có `FILE_SHARE_DELETE`, cho tới khi `GdipDisposeImage` chạy. Đây là hành vi có tài liệu của GDI+. `App.avatars` giữ `Image` suốt vòng đời ứng dụng.
- `actions.rs:91` gọi `remove_dir_all(profile)` **trước** `actions.rs:94` `a.avatars.remove`. Với tài khoản có ảnh đại diện, việc xóa luôn lỗi ở `avatar.png`. Lỗi chỉ được ghi log, tài khoản vẫn bị bỏ khỏi config, thư mục còn sót (có thể còn `Documents`, nơi Zalo lưu file nhận được, tùy thứ tự duyệt). Hộp thoại xác nhận lại hứa "dữ liệu ... sẽ bị xóa". Đây là vấn đề riêng tư.
- `avatar.rs:127`: `rename(tmp, avatar.png)` trong khi file cũ đang bị khóa sẽ lỗi "Access is denied", nên lần đổi ảnh thứ hai luôn thất bại (hộp thoại hiện chuỗi lỗi OS tiếng Anh).
- `actions.rs:360-362`: `avatar::remove` chạy trước `a.avatars.remove`, `remove_file` lỗi bị bỏ qua (`let _`), nên khi khởi động lại ảnh cũ hiện lại.
- Sửa: nạp ảnh vào bộ nhớ (đọc bytes → `SHCreateMemStream` → `GdipCreateBitmapFromStream`, hoặc clone bitmap rồi dispose bản từ file); luôn bỏ `Image` khỏi `avatars` trước khi xóa hoặc ghi đè; báo người dùng khi `remove_dir_all` lỗi và giữ mục cấu hình để thử xóa lại.

### H3. Mở lại từ khay khi cửa sổ đang thu nhỏ thì khung Zalo giữ kích thước 0×0 (C)
- Luồng: `WM_TRAY` → `with_app(|a| a.show_main())` (`main.rs:893, 900`) → `ShowWindow(SW_RESTORE)` (`main.rs:472`) gửi đồng bộ `WM_SIZE` → `main_proc` `WM_SIZE` → `with_app(|a| a.layout())` (`main.rs:739`) trả `None` vì `App` đang bị mượn. `layout` bị bỏ qua âm thầm.
- Trước đó, lúc thu nhỏ, `WM_SIZE(SIZE_MINIMIZED)` đã chạy `layout` với client rect 0×0 (`main.rs:249-251`), nên `pane` thành 0×0 và cửa sổ Zalo đang chọn cũng bị `place` về 0×0.
- Kịch bản: bấm nút thu nhỏ, sau đó bấm biểu tượng khay hoặc bấm thông báo của MultiZ. Cửa sổ hiện ra nhưng vùng Zalo trống cho tới khi người dùng đổi kích thước hoặc thu gọn thanh bên.
- Sửa: trong `WM_SIZE` bỏ qua `SIZE_MINIMIZED`; trong `show_main` gọi `self.layout()` thay cho `place_all()`, hoặc thực hiện `ShowWindow`/`SetForegroundWindow` ngoài `with_app`.

## MEDIUM

### M1. Lỗi đọc `config.json` khiến danh sách tài khoản bị ghi đè, không có bản sao (C)
- `config.rs:76`: mọi lỗi `read_to_string` (không chỉ `NotFound`), ví dụ file không phải UTF-8 sau khi sửa tay bằng Notepad ANSI, hoặc file đang bị antivirus khóa lúc khởi động, đều trả `Config::default()` mà **không** sao `config.broken.json`. Lần `save()` đầu tiên (`select`, `main.rs:350`, hoặc bất kỳ thao tác nào) ghi đè file gốc, nên danh sách tài khoản mất hẳn.
- Kể cả nhánh lỗi parse (có bản sao), các thư mục `profiles/<id>` vẫn mồ côi và không có cách nhận lại trong UI.
- Sửa: chỉ dùng mặc định khi `NotFound`; với lỗi khác, sao lưu file và đặt cờ chặn `save()` cho tới khi người dùng thao tác; cân nhắc dựng lại tài khoản từ các thư mục trong `profiles/`.

### M2. Chế độ gắn kèm: ẩn xuống khay (hoặc `--tray`) vẫn để cửa sổ Zalo nổi trên desktop (C)
- `main.rs:928` `ShowWindow(hwnd, SW_HIDE)`. Ẩn owner không ẩn cửa sổ owned; Windows chỉ tự ẩn chúng khi owner bị thu nhỏ. `place` (`embed.rs:78`) trả về sớm khi main ẩn, `tick` (`main.rs:516, 531-533`) cũng không xử lý nhánh main ẩn.
- Kịch bản: chế độ gắn kèm, bấm X (mặc định `close_to_tray = true`): MultiZ biến mất, cửa sổ Zalo đang chọn vẫn nằm giữa màn hình. Khởi động cùng Windows với `--tray`: `adopt` gán owner nhưng `place` bỏ qua, nên Zalo hiện ở vị trí riêng.
- Sửa: khi ẩn xuống khay (và khi `hidden_to_tray`), `park` mọi phiên ở chế độ gắn kèm; `show_main` đã gọi `place_all` để hiện lại.

### M3. Chế độ gắn kèm: thu nhỏ MultiZ có thể làm "đổi" cửa sổ chính của phiên sang cửa sổ phụ của Zalo (P)
- Khi owner bị thu nhỏ, Windows ẩn cửa sổ Zalo owned, nên `IsWindowVisible(s.hwnd) == 0`, còn `IsWindowVisible(main) != 0` (main thu nhỏ vẫn có `WS_VISIBLE`). Nhánh `main.rs:516-521` không kiểm tra `IsIconic(main)` và gọi `best_window(..., exclude=s.hwnd)`.
- Kịch bản: đang mở trình xem ảnh, cửa sổ cuộc gọi hoặc cửa sổ chat tách rời của Zalo (top-level, không owned, ≥250 px), người dùng thu nhỏ MultiZ. Trong 400 ms, cửa sổ phụ đó được `adopt`, còn cửa sổ chính bị `release` và không còn được theo dõi. Khi khôi phục, vùng pane hiện cửa sổ phụ.
- Sửa: bỏ qua nhánh "cửa sổ chính bị ẩn" khi `IsIconic(main) != 0`. Ở chế độ nhúng nên ưu tiên cửa sổ có tiêu đề kết thúc bằng "Zalo".

### M4. `hidden_to_tray` không được xóa khi cửa sổ được hiện bằng cách khác (C)
- Bản thứ hai hiện cửa sổ bằng `ShowWindow`/`SetForegroundWindow` (`main.rs:1029-1031`) mà không qua `show_main`, nên cờ `hidden_to_tray` vẫn là `true`. Khi đó `app_active()` (`main.rs:219`) luôn `false`.
- Kịch bản: MultiZ đang ở khay, người dùng bấm lại shortcut. Cửa sổ hiện ra, nhưng tài khoản đang xem vẫn bị đánh dấu "Có tin nhắn mới", vẫn bật toast, và watcher không coi tài khoản là `active`. Tình trạng kéo dài cho tới khi ẩn rồi mở lại từ khay.
- Sửa: xóa cờ trong `WM_SHOWWINDOW(wp=TRUE)` hoặc `WM_ACTIVATE`; hoặc bản thứ hai `PostMessage` một thông điệp riêng để bản đầu tự gọi `show_main`.

### M5. Đường dẫn Zalo.exe đã lưu trong bộ nhớ không bao giờ được tìm lại sau khi Zalo cập nhật hoặc gỡ (C)
- `main.rs:278-281`: chỉ gọi `find_zalo` khi `self.zalo` là `None`. `session::start` lỗi `CreateProcessW` (mã 2/3) không xóa `self.zalo`.
- Kịch bản: người dùng đã chọn tay đường dẫn tới thư mục phiên bản cụ thể, hoặc Zalo cài lại sang vị trí khác trong khi MultiZ đang chạy. Mọi lần "Mở Zalo"/"Thử lại" đều báo "mã lỗi 2" cho tới khi khởi động lại MultiZ (vẫn lỗi nếu `zalo_path` chọn tay đã hỏng, vì `find_zalo` chỉ dùng nó khi file còn tồn tại; trường hợp này có tự rơi về tự tìm).
- Sửa: khi `CreateProcessW` trả `ERROR_FILE_NOT_FOUND`/`ERROR_PATH_NOT_FOUND`, đặt `self.zalo = None`, gọi lại `find_zalo` và thử một lần.

### M6. Tiến trình Zalo có thể thoát khỏi Job, nên MultiZ mất quyền quản lý và có thể mở hai Zalo trên cùng profile (C cho đường mã, P cho tác nhân)
- `session.rs:162-164`: `AssignProcessToJobObject` lỗi chỉ ghi log rồi vẫn `ResumeThread`. Job rỗng, nên tick đầu tiên coi phiên đã thoát (`main.rs:490-503`) và `CloseHandle(job)`, trong khi Zalo vẫn chạy ngoài tầm quản lý. Bấm "Thử lại" sẽ mở Zalo thứ hai trên cùng profile.
- `session.rs:134` đặt `JOB_OBJECT_LIMIT_BREAKAWAY_OK`: bất kỳ tiến trình con nào tạo bằng `CREATE_BREAKAWAY_FROM_JOB` (một số luồng relaunch/cập nhật của Chromium/Electron dùng cờ này) đều thoát khỏi Job, gây cùng hậu quả và còn sống sót sau `TerminateJobObject` lúc xóa tài khoản.
- Sửa: khi Assign lỗi thì `TerminateProcess(pi.hProcess)` và trả `Err`. Cân nhắc bỏ `BREAKAWAY_OK` (Windows 8+ hỗ trợ job lồng nhau, sandbox Chromium không cần breakaway); nếu giữ, trước khi xóa profile cần kiểm tra không còn tiến trình nào mở file trong đó.

### M7. Tắt máy / thoát: Zalo bị giết cứng, không đóng êm (P)
- Không xử lý `WM_QUERYENDSESSION`/`WM_ENDSESSION` (`main.rs:726-947`). Cửa sổ Zalo nhúng là `WS_CHILD` nên không nhận các thông điệp phiên này (Windows chỉ gửi cho cửa sổ top-level). Khi MultiZ bị kết thúc, `KILL_ON_JOB_CLOSE` gây `TerminateProcess` mọi phiên.
- `exit_app` (`actions.rs:430`), `stop` (`session.rs:187`) cũng dùng `TerminateJobObject` ngay, không gửi `WM_CLOSE`.
- Kịch bản: tắt máy hoặc thoát MultiZ đúng lúc Zalo đang ghi CSDL tin nhắn hoặc tải file, có thể mất dữ liệu chưa ghi hoặc hỏng cache (thường phục hồi được, nhưng không bảo đảm).
- Sửa: trên `WM_ENDSESSION(TRUE)` và trong `exit_app`: `release` các cửa sổ về top-level, gửi `WM_CLOSE` tới cửa sổ chính mỗi phiên, chờ có giới hạn (ví dụ 3–5 giây, tận dụng `job_pids`), sau đó mới `TerminateJobObject`. Có thể dùng `ShutdownBlockReasonCreate` trong lúc chờ.

### M8. `remove_dir_all` trên đường dẫn ghép từ `id` chưa kiểm tra (C, cần config hỏng hoặc bị sửa)
- `actions.rs:90-91`: `profiles_dir().join(&acc.id)`. `id` đọc từ `config.json` không được kiểm tra. `id = ""` thì xóa **toàn bộ** `profiles`; `id = ".."` thì xóa cả `MultiZ` (config, log); `id` là đường dẫn tuyệt đối thì `join` thay hẳn gốc, nên xóa thư mục bất kỳ. Hai mục trùng `id` (sao chép tay) dùng chung profile: xóa một mục xóa dữ liệu cả hai, và `retain` (`actions.rs:95`) bỏ cả hai mục.
- Sửa: kiểm tra `id` khớp `^acc[0-9a-f]+$` khi load (loại mục sai, khử trùng lặp), và trước `remove_dir_all` khẳng định `dir.parent() == profiles_dir()`.

### M9. Chế độ nhúng: phiên không được chọn bị "hiện lại" cửa sổ cũ đã ẩn thay vì nhận cửa sổ mới (P)
- `main.rs:546-551`: với tài khoản không chọn, nếu `s.hwnd` bị ẩn thì `park` dùng `SWP_SHOWWINDOW` hiện lại chính cửa sổ đó; khác với nhánh tài khoản đang chọn (`main.rs:518`), nhánh này không tìm cửa sổ thay thế.
- Kịch bản: autostart nhiều tài khoản; Zalo của tài khoản không chọn hiện cửa sổ đăng nhập/splash (≥250 px) rồi ẩn nó (không hủy) khi mở cửa sổ chính. MultiZ ép cửa sổ cũ hiện lại, nên khi người dùng chọn tài khoản thì thấy màn hình cũ, và watcher đọc nhầm cửa sổ, cho tới khi bấm "Gắn lại cửa sổ".
- Sửa: áp dụng cùng logic `best_window(..., exclude)` cho nhánh không chọn trước khi `park`.

## LOW

### L1. Nhấp chuột phải biểu tượng khay có thể mở menu hai lần (P)
- `tray.rs:42` bật `NOTIFYICON_VERSION_4`; ở v4 shell gửi cả `WM_RBUTTONUP` lẫn `WM_CONTEXTMENU`. `main.rs:902` xử lý cả hai, nên `tray_menu` chạy hai lần (lần hai lồng trong vòng lặp `TrackPopupMenu` của lần đầu, hoặc mở lại ngay sau khi đóng). `WM_LBUTTONUP` + `NIN_SELECT` cũng gọi `show_main` hai lần (vô hại).
- Sửa: với v4 chỉ xử lý `WM_CONTEXTMENU` và `NIN_SELECT`/`NIN_KEYSELECT`.

### L2. Thao tác bị bỏ âm thầm do mượn lồng trong `show_main` (C)
- `main.rs:470-474` chạy trong `with_app`. `WM_ACTIVATE`/`WM_SETFOCUS` lồng gọi `request_focus` → `hwnd_main()` (`actions.rs:6`) trả `null`, nên không có `WM_APP_FOCUS`. Ở chế độ attached, `WM_WINDOWPOSCHANGED` (`main.rs:743`) cũng bị bỏ qua. Mở từ khay bằng chuột trái thì Zalo nhúng không nhận bàn phím cho tới khi người dùng bấm vào.
- Sửa: gom chung với H3: gọi Win32 ngoài phạm vi mượn, rồi `PostMessageW(WM_APP_FOCUS)` ở cuối `show_main`.

### L3. Cắt chuỗi UTF-16 giữa cặp surrogate (C)
- `tray.rs:11-16` cắt theo số đơn vị UTF-16 (`szInfoTitle` 64, `szInfo` 256, `szTip` 128). `short(text, 200)` tính theo ký tự, nên chuỗi có emoji (phổ biến trong tên và tin Zalo) có thể vượt 256 đơn vị và bị cắt giữa surrogate, hiện ký tự lỗi.
- Sửa: lùi `n` nếu `v[n-1]` là high surrogate.

### L4. Phiên thoát để lại trạng thái tin mới (C)
- `main.rs:562-570` chỉ xóa `sessions`, không xóa `has_new`, `pending`, `flash_pending` như `stop` (`main.rs:311-313`). Tooltip khay và chấm đỏ tiếp tục báo "tin mới" của tài khoản đã tắt; `pending` có thể vẫn gửi toast sau khi phiên đã chết.
- Sửa: dùng chung phần dọn dẹp của `stop`.

### L5. `stop` chặn luồng UI tới 3 giây; "Khởi động lại" có thể mở Zalo khi bản cũ chưa thoát hẳn (P)
- `session.rs:189-195` ngủ trong khi giữ mượn `App`; không bơm thông điệp. Nếu sau 3 giây job chưa rỗng, `start` (`actions.rs:477-478`, `main.rs:349-350`) vẫn chạy. Zalo mới thấy khóa profile của bản cũ, tự thoát, và MultiZ hiện lỗi "Zalo đã tự thoát ngay sau khi mở".
- Sửa: chờ bằng `WaitForSingleObject` trên handle tiến trình hoặc dùng completion port của Job, và hoãn `start` tới khi job rỗng (qua tick).

### L6. `Config::save` không bền khi mất điện, lỗi bị bỏ qua (C)
- `config.rs:84-85`: `fs::write` không `sync_all` trước `rename`; lỗi `write`/`rename` (file chỉ đọc, bị khóa) bị nuốt, nên thay đổi mất âm thầm. Mất điện ngay sau khi lưu có thể để lại `config.json` rỗng, dẫn về M1.
- Sửa: `File::create` + `write_all` + `sync_all`, rồi `rename`; ghi log khi lỗi.

### L7. Xoay vòng log không an toàn giữa hai luồng (C)
- `util.rs:59-63` được gọi đồng thời từ luồng UI và watcher. Hai luồng cùng thấy file > 1 MB thì lần `rename` thứ hai ghi đè `log.old.txt` bằng file mới gần rỗng, làm mất log cũ.
- Sửa: bọc `log` bằng `Mutex` tĩnh.

### L8. Riêng tư trong log (C)
- Nội dung tin nhắn **không** được ghi log (`main.rs:398-402` chỉ ghi "có nội dung"). Tuy vậy, tiêu đề cửa sổ Zalo được ghi nguyên văn (`embed.rs:146-152`, `main.rs:519`); cửa sổ chat tách rời hoặc cửa sổ cuộc gọi có tiêu đề là tên người liên hệ. Tên tài khoản và đường dẫn profile (chứa tên người dùng Windows) cũng được ghi.
- Sửa: chỉ ghi class và độ dài tiêu đề, hoặc chỉ ghi khi tiêu đề khớp mẫu `(n) Zalo`/`Zalo`.

### L9. Enter khi focus đang ở nút "Hủy" lại thực hiện OK (C)
- `dialog.rs:317-319` chặn mọi `WM_KEYDOWN VK_RETURN` trước `IsDialogMessageW` và gọi `do_ok`.
- Sửa: chỉ gọi `do_ok` khi `GetFocus()` không phải nút Hủy.

### L10. Tách profile phụ thuộc biến `%USERPROFILE%` trong User Shell Folders (P)
- `session.rs:105-114`: tách dữ liệu dựa vào việc Windows mở rộng `User Shell Folders` theo môi trường của tiến trình. Máy có Folder Redirection (AppData chuyển sang đường dẫn tuyệt đối hoặc mạng) hoặc OneDrive Known Folder Move cho Documents thì mọi phiên dùng chung dữ liệu hoặc thư mục "Zalo Received Files", và hiện không có cảnh báo nào.
- Sửa: lúc khởi động đọc `HKCU\...\User Shell Folders\AppData` và `Personal`; nếu không chứa `%USERPROFILE%` thì cảnh báo người dùng.

## Đã kiểm tra, không thấy lỗi
- `job_pids` (`session.rs:47-65`): bố cục `JOBOBJECT_BASIC_PROCESS_ID_LIST` trên x64 đúng, chặn `n.min(2048)`.
- Buffer UTF-16: `util::w` luôn có NUL; `from_w` dừng ở NUL; `copy` trong `tray.rs` luôn kết thúc NUL; buffer của `RegEnumKeyExW`, `GetWindowTextW`, `GetOpenFileNameW` đủ kích thước.
- Kích thước struct: `cbSize`/`cb`/`lStructSize` đều dùng `size_of` của struct tương ứng.
- GDI/GDI+ trong vẽ: brush, pen, path, graphics, bitmap double buffer đều được giải phóng; font cũ được giải phóng khi đổi DPI; font hộp thoại được xóa sau `DestroyWindow`.
- `split_home` không panic với đường dẫn đa byte (byte 1 là `:` thì byte 0 là ASCII).
- `TaskbarCreated` được xử lý; tọa độ âm (đa màn hình) dùng `i16` sign-extend đúng ở `lo`/`hi`, `WM_CONTEXTMENU` và `TTM_TRACKPOSITION`.
- Watcher: sự kiện cho tài khoản đã xóa bị loại ở `new_message` (`main.rs:395`); Mutex không thể bị poison vì `panic = "abort"`.
- `remove_dir_all` của Rust std (≥1.58) không đi theo junction/symlink, nên không xóa lan ra ngoài qua link.

## Câu hỏi còn mở
- Zalo PC relaunch/cập nhật có dùng `CREATE_BREAKAWAY_FROM_JOB` không? Câu trả lời quyết định nên bỏ `BREAKAWAY_OK` (M6) hay không; cần thử thực tế bằng Process Explorer.
- Thứ tự Explorer gửi `WM_RBUTTONUP`/`WM_CONTEXTMENU` ở v4 (L1) nên được xác nhận trên Windows 11.
