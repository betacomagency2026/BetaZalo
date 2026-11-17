# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Ràng buộc cố định

1. Câu từ output ngắn gọn, không dài dòng. Văn phong học thuật, chuẩn mực.
2. Chỉ ghi vào tài liệu dự án những ràng buộc mà người dùng nói rõ là cố định.
3. Không viết comment trong code. Quy tắc này ưu tiên hơn mọi hướng dẫn chung yêu cầu viết comment.
4. Cấu trúc thư mục logic, rõ ràng, khoa học; có thể áp dụng Package by Feature / Module, MVC / MVCS.
5. Commit khi hoàn thành một module hoặc cuối phiên làm việc, rồi push lên `https://github.com/betacomagency2026/BetaZalo`. Tác giả commit: `nottramdw`. Email: nguyenhuungocduonghd@gmail.com
6. Cuối phiên, cập nhật `docs/tien-do.md`.

## Quy trình phiên

- Đầu phiên: đọc `docs/tien-do.md`, `docs/quyet-dinh.md` và plan đang làm trong `plans/`. Nêu mục tiêu phiên trong một câu rồi làm ngay, không hỏi lại việc đã chốt.
- Trong phiên: chỉ làm phase đã chọn. Việc phát sinh ghi vào "Việc tiếp theo" của `docs/tien-do.md`, không làm ngay.
- Mỗi quyết định chốt ghi một dòng vào `docs/quyet-dinh.md`.
- Cuối phiên: build và lint đạt, cập nhật trạng thái phase trong plan, cập nhật nhật ký và việc tiếp theo trong `docs/tien-do.md`, commit, push.
- Làm việc tự chủ: chỉ hỏi khi lựa chọn làm thay đổi phạm vi hoặc không thể suy ra từ tài liệu dự án. Khi hỏi, nêu phương án đề xuất và làm trước mọi phần không phụ thuộc câu trả lời.

## Tổng quan

MultiZ (Sudo Software, MIT, v0.3.4) chạy nhiều tài khoản Zalo PC gốc trên một máy; mỗi tài khoản một thư mục dữ liệu riêng, không sửa và không chèn mã vào Zalo. Hai bản độc lập, không chung mã:

- `windows/`: Rust, Win32 thuần, không framework UI, xuất một file `.exe`.
- `mac/`: Swift/AppKit, ứng dụng thanh menu, không có cửa sổ chính.

`LICENSE` chỉ đặt ở gốc. `README.md` gốc là trang tổng quan; README và PRIVACY chi tiết nằm trong từng thư mục nền tảng.

## Lệnh

Windows, chạy trong `windows/`:

```
cargo build --release      # target/release/multiz.exe
cargo fmt --check          # rustfmt.toml: max_width 120
cargo clippy --release
```

- `build.rs` biên dịch `res/app.rc` (icon, manifest, thông tin phiên bản). Target MSVC dùng `rc.exe`: biến `RC`, nếu không có thì bản Windows SDK mới nhất trong `Windows Kits\10\bin`. Target GNU dùng `windres`: biến `WINDRES`, mặc định `x86_64-w64-mingw32-windres`.
- Không có test tự động.

macOS, chạy trong `mac/` (cần macOS và `xcode-select --install`):

```
bash build.sh              # build/MultiZ.app, build/MultiZ-mac.zip (universal, ký ad-hoc)
```

Số phiên bản nằm ở: `windows/Cargo.toml`; `windows/res/app.rc` (FILEVERSION, PRODUCTVERSION, chuỗi); `mac/Resources/Info.plist` (CFBundleShortVersionString, CFBundleVersion); giá trị dự phòng trong `mac/Sources/AppDelegate.swift`; `mac/README.md`. Nâng phiên bản phải sửa đồng bộ.

## Kiến trúc Windows

**Tách tài khoản** (`session.rs`): `CreateProcessW` chạy Zalo.exe ở trạng thái suspended, khối môi trường ghi đè `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, `TEMP`, `TMP`, `HOMEDRIVE`, `HOMEPATH` trỏ vào `%LOCALAPPDATA%\MultiZ\profiles\<id>`; tiến trình được gán vào Job Object (`KILL_ON_JOB_CLOSE`) rồi resume. PID của phiên luôn lấy từ Job Object (`Session::pids`) vì Zalo (Electron) sinh nhiều tiến trình con; job rỗng nghĩa là phiên đã thoát. Cửa sổ chính chọn bằng `best_window`: ưu tiên đang hiện, class `Chrome_WidgetWin_1`, diện tích lớn nhất.

**Trạng thái và gọi lồng**: toàn bộ trạng thái nằm trong một `App` ở `thread_local!` `RefCell` (`main.rs`), chỉ truy cập qua `with_app`; hàm này trả `None` thay vì panic khi đang mượn lồng. Hộp thoại, `TrackPopupMenu`, `MessageBoxW` chạy vòng lặp thông điệp riêng nên window procedure bị gọi lồng. Các hàm trong `actions.rs` vì vậy chỉ mượn `App` trong phạm vi ngắn và không giữ mượn khi mở hộp thoại; thao tác mới phải theo cùng mẫu. Thông điệp bố cục (`WM_SIZE`, `WM_MOVE`, `WM_DPICHANGED`) đến khi `App` đang bị mượn thì được hoãn qua `defer_layout` (`WM_APP_RELAYOUT`).

**Tắt phiên**: `App::stop`/`restart` gọi `TerminateJobObject` rồi chuyển job vào `stopping`, không chờ trên luồng UI; `tick` (`reap_stopping`) đóng handle khi job rỗng, mở lại nếu cần; quá `STOP_TIMEOUT` (10 giây) thì báo lỗi, không mở lại. `start` khi id đang trong `stopping` chỉ đặt cờ mở lại. Cửa sổ đứng hình (`IsHungAppWindow`) bị bỏ qua khi đặt vị trí và trao focus.

**Vòng `App::tick`** (timer 400 ms): phát hiện phiên đã thoát; gắn cửa sổ mới tìm được (`adopt`); giữ vị trí cửa sổ; đọc số chưa đọc từ tiêu đề dạng `(3) Zalo`; gửi thông báo đang chờ (gom tin trong `NOTIFY_COOLDOWN` 8 giây); chuyển danh sách cửa sổ cho watcher; mở lần lượt các tài khoản `autostart` cách nhau 1,5 giây; chỉ vẽ lại khi chuỗi chữ ký trạng thái thay đổi.

**Hai chế độ hiển thị** (`embed.rs`, `Config::embed`):

- Nhúng: bỏ style khung, `SetParent` vào cửa sổ con `pane`. Tài khoản không chọn được đặt ở tọa độ âm nhưng vẫn hiển thị (`park`) để Chromium tiếp tục vẽ và UI Automation đọc được tin mới. Cửa sổ con không nhận kích hoạt từ Windows, nên MultiZ tự gửi `WM_NCACTIVATE`, `WM_ACTIVATE` và `SetFocus` qua `WM_APP_FOCUS` (`activate_child`).
- Gắn kèm: đặt owner là cửa sổ chính, phủ đúng vùng `pane` và đi theo khi di chuyển; tài khoản không chọn bị ẩn.

`release` khôi phục style gốc (`orig_style`, `orig_exstyle`) trước khi đổi chế độ hoặc gắn lại cửa sổ.

**Phát hiện tin mới** có hai nguồn:

- `watcher.rs`: luồng riêng (COM MTA), mỗi 2,5 giây đọc các phần tử Text qua UI Automation trong vùng danh sách hội thoại (x từ 56 đến min(480, 55% bề rộng), y > 56, nhân theo DPI). Chuỗi được chuẩn hóa (bỏ từ chỉ thời gian tiếng Việt/Anh và chữ số) rồi so tập dòng với lần đọc trước. Có 1–4 dòng mới không bắt đầu bằng `Bạn:`/`You:` thì sinh sự kiện; nhiều hơn coi là tải lại danh sách. Giao tiếp với luồng UI qua `Mutex` và `PostMessageW(WM_WATCH_EVENT)`. Heuristic này phụ thuộc bố cục và chuỗi giao diện của Zalo.
- Shell hook `HSHELL_FLASH`: Zalo nhấp nháy taskbar mà watcher chưa bắt được trong 3 giây thì báo chung "Có tin nhắn mới".

**Vẽ giao diện** (`ui.rs`, `gfx.rs`): tự vẽ hoàn toàn, double buffer. `Canvas` vẽ hình bằng GDI+ ngay; chữ được hoãn tới `finish()` và vẽ bằng GDI (ClearType) đè lên. Vùng bấm (`hits`, `pane_hits`) được dựng lại trong mỗi lần vẽ và tra bằng `hit_at`, nên nút mới phải được đẩy vào danh sách này khi vẽ. Kích thước theo DPI qua `App::s`.

**Thông điệp tự định nghĩa**: `WM_APP+10` khay hệ thống (`tray.rs`), `WM_APP+11` watcher, `WM_APP+20` trao focus, `WM_APP+21` bố cục bị hoãn.

**Tổ chức module**: `actions`, `embed`, `tooltip`, `ui` dùng `use super::*` và truy cập trực tiếp trường của `App`; các module còn lại độc lập, import qua `crate::`. Crate `windows` chỉ dùng cho COM/UI Automation trong `watcher.rs`; phần còn lại dùng `windows-sys`.

**Dữ liệu và tương thích ngược**: `config.json` dùng serde, mọi trường có giá trị mặc định; lưu nguyên tử (ghi `.tmp`, `sync_all`, rồi đổi tên); file hỏng được sao sang `config.broken-<thời điểm>.json`; không đọc được file (khác lỗi không tồn tại) hoặc không sao lưu được thì `read_only` và `save()` không ghi. Xóa tài khoản chỉ xóa thư mục khi id hợp lệ (`util::is_safe_id`) và không trùng. `util::data_root` đổi tên thư mục cũ `Zaloo`/`MultiZalo` thành `MultiZ`; `migrate_autorun` chuyển khóa Run `Zaloo` sang `MultiZ`. Chỉ một bản chạy cùng lúc (mutex `Local\SudoSoftware.MultiZ.SingleInstance`); tham số `--tray` khởi động ẩn dưới khay.

## Kiến trúc macOS

Không nhúng cửa sổ, không phát hiện tin mới; chỉ mở, tắt và chuyển phiên Zalo từ menu trên thanh menu.

- `ZaloLauncher.swift`: chạy executable của Zalo.app với `CFFIXED_USER_HOME`, `HOME`, `TMPDIR` trỏ vào `~/Library/Application Support/MultiZ/profiles/<id>`, bỏ `__CFBundleIdentifier` và `XPC_SERVICE_NAME`. `Library/Keychains` trong profile là symlink tới Keychain thật. PID ghi vào `profiles/<id>/zalo.pid` để `restore()` nhận lại phiên khi MultiZ khởi động lại. Chuyển tài khoản gồm `activate` và Apple Event reopen (cần quyền Automation).
- `Store.swift`: `config.json` gồm `accounts` và `zaloPath`; cờ `reopen` tương ứng `autostart` của bản Windows. Xóa tài khoản chuyển profile vào Thùng rác, trong khi bản Windows xóa hẳn.
- `Avatar.palette` phải trùng `COLORS` trong `windows/src/config.rs`.
- Mở cùng máy bằng LaunchAgent `~/Library/LaunchAgents/vn.sudo.multiz.plist`.

