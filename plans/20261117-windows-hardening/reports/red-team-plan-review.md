# Red-team plan gia cố bản Windows

Phạm vi: `plan.md`, `phase-01` đến `phase-06`, đối chiếu mã `windows/src/*.rs` (HEAD `87bad24`) và tài liệu Win32/GDI+/UIA. Chỉ đọc; không chạy build (hook chặn thư mục target ngoài repo).

Mức độ: **blocker** = phải sửa plan trước khi làm phase; **major** = sai hoặc thiếu gây hồi quy/không đạt yêu cầu phase; **minor** = chi tiết triển khai.

## Blocker

### B1. Phase 4: Khởi động lại rồi xóa tài khoản làm Zalo chạy lại trên tài khoản đã xóa
- Plan: `tick` gặp `Stopping { restart: true }` thì gọi `start`; `delete_account` gọi `stop` rồi `wait_exit`.
- Mã: `App::start` (`main.rs:274-298`) không kiểm tra `cfg.index_of(id)`; `session::start` gọi `prepare_profile` (`session.rs:92`) nên tạo lại thư mục profile.
- Kịch bản: Zalo đứng hình, người dùng bấm "Khởi động lại" (`actions.rs:348-352` hoặc `476-479`: `stop` rồi `start`, nên `restart = true`), phiên cũ chưa thoát, người dùng chuyển sang "Xóa tài khoản". `delete_account` xóa profile và mục config, nhưng mục `stopping` vẫn còn. Tick kế tiếp gọi `start`, tạo lại `profiles/<id>` và mở một Zalo không có trong danh sách (UI không hiển thị, chỉ tắt được bằng thoát MultiZ).
- Sửa plan: `delete_account` lấy mục `stopping` ra (chờ, `CloseHandle`, bỏ cờ restart) trước khi xóa; `App::start` trả về sớm khi `cfg.index_of(id)` là `None`; nhánh restart trong `tick` cũng kiểm tra tài khoản còn tồn tại.

## Major

### M-1. Phase 4: Hết 5 giây vẫn mở phiên mới, tạo lại đúng lỗi L5
- Plan: "nếu `job_pids` rỗng **hoặc quá 5 giây** thì `CloseHandle`; nếu `restart` thì gọi `start`". Yêu cầu của chính phase: "chỉ mở phiên mới khi phiên cũ đã thoát hẳn".
- Hai điều này mâu thuẫn. Tiến trình không thoát sau `TerminateJobObject` (I/O treo, antivirus giữ handle) vẫn khóa profile; Zalo mới sẽ tự thoát và hiện lỗi "Zalo đã tự thoát ngay sau khi mở" (`main.rs:491-494`), đúng kịch bản L5.
- Sửa plan: khi quá hạn, không `start`; đặt `failed[id]` với thông báo "Phiên cũ chưa thoát hẳn, hãy thử lại sau" và giữ job cho tới khi rỗng (hoặc đóng job nhưng không tự mở lại).

### M-2. Phase 4: Phạm vi quá hẹp so với yêu cầu "Zalo đứng hình không làm MultiZ treo"
- Plan chỉ đổi `ShowWindow` trong `stop`. Các lời gọi đồng bộ chéo tiến trình khác vẫn còn: `place`/`park` (`embed.rs:76,83,108,110`), `ShowWindow(SW_RESTORE)` trong tick (`main.rs:529`), `reattach` (`main.rs:331`).
- Kịch bản: tài khoản A đứng hình; người dùng bấm chọn A để xem chuyện gì xảy ra. `select` → `place_all` → `place(A)` → `SetWindowPos` đồng bộ tới luồng đang treo, nên MultiZ treo trước khi người dùng kịp bấm "Tắt". Ở chế độ gắn kèm, chỉ cần đổi kích thước MultiZ (`WM_SIZE`/`WM_WINDOWPOSCHANGED` → `place_all`) là treo.
- Ở chế độ nhúng, `SetParent` chéo tiến trình gắn hàng đợi nhập liệu (audit W5). Khi đó `SWP_ASYNCWINDOWPOS` không có tác dụng (tài liệu: cờ này chỉ đưa vào hàng đợi khi hai luồng có hàng đợi nhập liệu khác nhau), và một Zalo bị tạm dừng có thể chặn cả chuột, phím của MultiZ.
- Sửa plan: thêm vào phase 4: (1) bỏ qua `place`/`park` cho cửa sổ có `IsHungAppWindow != 0`; (2) chế độ gắn kèm dùng `SWP_ASYNCWINDOWPOS` và `ShowWindowAsync`; (3) ghi rõ giới hạn của chế độ nhúng. Kiểm thử mục 4 phải chạy cả hai chế độ, với cả hai trường hợp: tài khoản treo đang chọn và tài khoản treo chưa chọn (bấm chọn nó rồi mới tắt; tắt bằng menu chuột phải).

### M-3. Phase 2 H1: Giả định về `LR_SHARED` trái với tài liệu
- Plan (Rủi ro): "`LR_SHARED` với kích thước khác vẫn cho icon dùng chung theo cặp (resource, kích thước)".
- Tài liệu `LoadImageW`: "This function finds the first image in the cache with the requested resource name, **regardless of the size requested**" và "Do not use LR_SHARED for images that have non-standard sizes".
- Mã: `main.rs:1040` đã gọi `LoadIconW(hinst, 1)`. Hàm này nạp ở chế độ shared với kích thước `SM_CXICON`, nên cache đã có bản 32 px trước khi gọi `tray::add`. Theo tài liệu, `LoadImageW(..., SM_CXSMICON, LR_SHARED)` có thể trả bản 32 px; icon khay sẽ bị thu nhỏ, mờ ở DPI cao. Hành vi thực tế trên Windows 10/11 có thể khớp cả kích thước, nhưng plan đang dựa vào điều mà tài liệu nói ngược.
- Sửa plan: nạp hai icon (nhỏ, lớn) **một lần**, không dùng `LR_SHARED`, giữ trong `App` (hoặc `thread_local`), dùng lại ở `add` (cả khi `TaskbarCreated`) và `notify`. Phương án thay thế: `DestroyIcon` ngay sau `Shell_NotifyIconW`. Kiểm thử: icon khay và icon toast sắc nét ở 150%; USER objects không tăng sau khoảng 50 thông báo.

### M-4. Phase 5: `IUIAutomation2` không lấy được từ `CUIAutomation`
- Mã: `watcher.rs:134` tạo `CUIAutomation`. Tài liệu `CUIAutomation8`: lớp này hiện thực `IUIAutomation2` (Windows 8+). Bản mô phỏng của Wine trả `E_NOINTERFACE` cho `IUIAutomation2` khi đối tượng không phải `CUIAutomation8`.
- Nếu chỉ `cast::<IUIAutomation2>()` rồi `let _ =` lỗi, timeout sẽ không bao giờ được áp dụng mà không ai biết. Ngoài ra, trong windows 0.58 phương thức có tên `SetTransactionTimeout`/`SetConnectionTimeout`, và `cast` cần import trait `Interface`.
- Sửa plan: tạo `CUIAutomation8`, nếu lỗi thì rơi về `CUIAutomation`; ghi log một lần cho biết timeout có được áp dụng không. Đổi CLSID có thể đổi hành vi UIA core, nên phải kiểm thử lại mục 6.

### M-5. Phase 5: Timeout giao dịch 2 giây có thể cắt các lần đọc hợp lệ; plan bỏ bước đo
- Mặc định của UIA: kết nối 2 s, giao dịch 20 s. `FindAllBuildCache(TreeScope_Descendants)` (`watcher.rs:92`) duyệt cả khung tin nhắn. Với hội thoại dài trên máy yếu, một lần đọc có thể vượt 2 s, gây `UIA_E_TIMEOUT` liên tục, và tài khoản đó mất phát hiện tin hoàn toàn (lỗi chỉ ghi log một lần, `watcher.rs:161-164`).
- Audit W4 yêu cầu đo trước (thời gian đọc p50/p95, số phần tử). Plan đã bỏ bước này.
- Sửa plan: thêm log thời gian đọc và `arr.Length()` mỗi khoảng 100 vòng; đặt timeout giao dịch theo p95 đo được (gợi ý ≥ 5 s); đếm và ghi log số lần timeout. Nhánh `FindAll` dự phòng (`watcher.rs:94`) cũng chịu timeout này.

### M-6. Phase 5: Bỏ đọc tài khoản `active` làm mất tin, trái với nhận định "giữ hành vi hiện tại"
- Mã hiện tại (`watcher.rs:184-187`): khi `active`, mỗi vòng vẫn đọc và cập nhật `prev`. Khi tài khoản vừa rời trạng thái active, lần đọc sau so với mốc mới nhất, nên tin đến sau thời điểm rời đi vẫn được báo.
- Plan: khi `active` thì không đọc, đặt `seen_once = false`. Lần đọc đầu sau khi rời đi chỉ lấy mốc, nên tin đến trong khoảng đó (tới 2,5 s cộng thời gian đọc mọi tài khoản) bị nuốt.
- `active` = `app_active() && đang chọn` (`main.rs:576, 621`), nên trạng thái này đổi mỗi lần người dùng Alt+Tab sang ứng dụng khác hoặc đóng xuống khay. Đó lại là lúc người dùng cần thông báo nhất.
- Sửa plan: bỏ mục này (tiết kiệm chỉ một tài khoản mỗi vòng), hoặc giữ đọc nhưng thưa hơn khi active, và đọc ngay ở vòng đầu tiên sau khi rời active để mốc cách thời điểm rời đi không quá 1 vòng. Thêm kiểm thử: chuyển sang ứng dụng khác rồi gửi tin ngay trong 1–2 giây, phải có thông báo.

### M-7. Phase 3 M2: Ẩn Zalo ở chế độ gắn kèm chưa được chứng minh an toàn cho watcher; đường `--tray` chưa được bao phủ
- Plan (Rủi ro) khẳng định UIA vẫn đọc được cửa sổ Chromium bị ẩn, nhưng không đưa bằng chứng. Thiết kế hiện tại cho thấy điều ngược lại: CLAUDE.md và `main.rs:547-548` giải thích rằng `park` phải giữ cửa sổ "đang hiển thị" để Zalo tiếp tục cập nhật và UIA đọc được. M2 ẩn đúng tài khoản đang chọn khi MultiZ ở khay, tức kịch bản "chạy dưới khay để nhận thông báo".
- `--tray`: lúc khởi động chưa có cửa sổ Zalo nào. Cửa sổ được `adopt` sau đó trong `tick` (`main.rs:511-514`), rồi `place` trả về sớm vì main đang ẩn (`embed.rs:78-80`). Nếu chỉ "ẩn khi WM_CLOSE / --tray", cửa sổ gắn sau đó vẫn nổi trên màn hình.
- Sửa plan: (1) trong `adopt`/`place` ở chế độ gắn kèm, khi main ẩn thì `park` bằng `ShowWindowAsync(SW_HIDE)`; (2) thêm kiểm thử: chế độ gắn kèm, MultiZ ở khay, gửi tin tới **tài khoản đang chọn** và tới tài khoản không chọn, cả hai phải có thông báo; (3) nếu kiểm thử thất bại, chuyển phương án sang đặt cửa sổ ra ngoài virtual screen (`SM_XVIRTUALSCREEN`) mà không ẩn. Có thể dùng kết quả hiện có để đối chiếu: ở chế độ gắn kèm, tài khoản không chọn đã bị `SW_HIDE` (`embed.rs:110`). Nếu các tài khoản này hiện vẫn được báo tin, M2 an toàn.

### M-8. Phase 2 M1: Bản sao lưu không tạo được đúng lúc cần nhất; không chặn `save()`
- Khi `read_to_string` lỗi vì file bị khóa (antivirus quét lúc khởi động cùng Windows), `fs::copy` cũng lỗi, nên không có bản sao lưu. Lần `save()` đầu tiên (`select` gọi `cfg.save()` ở `main.rs:350`, `toggle_sidebar`, `WM_CLOSE`…) ghi đè `config.json` bằng danh sách rỗng.
- Audit M1 đã đề xuất cờ chặn `save()`; plan bỏ qua.
- Sửa plan: với lỗi không phải `NotFound`, thử đọc lại 3 lần cách nhau 200 ms; nếu vẫn lỗi hoặc sao lưu thất bại thì đặt cờ `save_blocked` (không ghi đè cho tới khi người dùng thêm hoặc xóa tài khoản) và báo bằng `msgbox`. Kiểm thử: mở `config.json` với share-mode độc quyền rồi khởi động MultiZ; thử thêm với file ANSI.

### M-9. Phase 2: Xóa tài khoản vẫn âm thầm để lại dữ liệu khi gặp nguyên nhân khóa khác
- Plan chỉ xử lý nguyên nhân do avatar. `remove_dir_all` vẫn có thể lỗi vì Zalo chưa thoát hết (hết hạn `wait_exit`), tiến trình con đã breakaway (`JOB_OBJECT_LIMIT_BREAKAWAY_OK`, `session.rs:134`, không nằm trong job nên `wait_exit` không thấy), antivirus, hoặc Explorer đang mở thư mục. Khi đó mã vẫn chỉ ghi log (`actions.rs:91-93`) rồi bỏ mục khỏi config (`actions.rs:95`), trái với câu hứa trong hộp thoại xác nhận (`actions.rs:82`).
- Sửa plan: khi `remove_dir_all` lỗi, giữ mục config (hoặc đánh dấu "chờ xóa" để thử lại lúc khởi động) và báo người dùng kèm đường dẫn thư mục còn sót.

## Minor

| # | Phase/mục | Vấn đề và bằng chứng | Đề xuất |
|---|---|---|---|
| m1 | 2 / H1 | Phương án `OnceLock` không biên dịch: trong windows-sys 0.59, `HICON` là `*mut c_void`, không `Sync` (mã đã dùng `is_null()`/`null_mut()` cho handle) | Lưu trong `App` hoặc `thread_local! Cell<isize>` |
| m2 | 2 / H2 | Không cần stream: chỉ cần chép sang bitmap độc lập là đã bỏ được khóa. Nếu vẫn dùng `SHCreateMemStream`, windows-sys không có wrapper COM, phải gọi `Release` qua vtable thủ công. GDI+ yêu cầu giữ stream suốt vòng đời bitmap nguồn | `GdipCreateBitmapFromFile` → `GdipCreateBitmapFromScan0(w,h,0,PARGB,null)` → vẽ → `GdipDisposeImage(nguồn)`. Nếu giữ stream: dispose nguồn trước rồi mới Release; `scan0` phải là `null` để GDI+ sở hữu bộ nhớ |
| m3 | 2 / M8 | Chỉ kiểm tra id khi xóa; `start` (`main.rs:286` → `prepare_profile`), "Mở thư mục dữ liệu" (`actions.rs:384-385`) và `avatar::path` cũng ghép id. Id tuyệt đối hoặc `..` vẫn tạo thư mục ngoài `profiles` | Kiểm tra id lúc `Config::load`: loại mục sai, khử trùng lặp, ghi log |
| m4 | 3 / M5 | `session::start` trả `Err(String)` (`session.rs:160`); muốn biết "không tìm thấy file" phải phân tích chuỗi | Trả mã lỗi có cấu trúc; khi gặp 2/3 thì đặt `zalo = None`, `find_zalo` và thử lại một lần (như audit đề xuất) |
| m5 | 3 / H3 | Vẫn chạy `layout` với client 0×0 khi thu nhỏ (`main.rs:738-740`): pane về 0×0, mọi cửa sổ nhúng bị đổi kích thước hai lần mỗi lần thu nhỏ/khôi phục | Bỏ qua `SIZE_MINIMIZED`. Handler `WM_APP_RELAYOUT` xóa cờ trước, nếu `with_app` vẫn `None` thì post lại. Với DPI: đọc lại `GetDpiForWindow` trong handler thay vì dựa vào `wParam` đã mất |
| m6 | 3 / L4 | `stop` cũng không xóa `flash_pending` (`main.rs:311-313`); `new_message` không kiểm tra phiên còn chạy, nên flash cũ vẫn có thể bật `has_new` cho tài khoản đã tắt | Gom một hàm dọn dẹp dùng chung cho `stop`, phiên chết và `delete_account` |
| m7 | 3 / M2 | Ẩn cửa sổ Zalo bằng `ShowWindow` đồng bộ lại tạo thêm điểm treo kiểu W2 | Dùng `ShowWindowAsync` |
| m8 | 3 (hoãn M3) | Chặn bằng `IsIconic(main)` là sửa thuần mã, đúng theo cấu trúc: khi owner thu nhỏ, Windows ẩn cửa sổ owned, nên nhánh `main.rs:516` hiểu nhầm là Zalo tự ẩn. Không cần thử với Zalo thật | Đưa phần chặn `IsIconic(main) == 0` vào phase 3 (cùng nhánh với M2); phần ưu tiên tiêu đề vẫn hoãn |
| m9 | 4 (hoãn M6) | `AssignProcessToJobObject` lỗi vẫn `ResumeThread` (`session.rs:162-165`), dẫn tới hai Zalo trên một profile, trái mục tiêu phase 4. Sửa thuần mã | Đưa phần "Assign lỗi thì `TerminateProcess` + `Err`" vào phase 4; chỉ hoãn phần bỏ `BREAKAWAY_OK` |
| m10 | 4 | Chữ ký vẽ lại (`main.rs:636-648`) không có `stopping`, nên trạng thái "Đang tắt…" không tự đổi khi phiên thoát xong | Thêm trạng thái `stopping` vào `sig` |
| m11 | 4 | "Khởi động lại" rồi "Tắt" trong lúc đang chờ: plan không xóa `restart`, nên phiên vẫn tự mở lại | `stop` trên id đang `stopping` thì đặt `restart = false` |
| m12 | 4 / exit_app | Plan viết "**giữ** chờ đồng bộ", nhưng `exit_app` hiện không chờ (`actions.rs:424-434`). Chờ thêm không có lợi vì thoát tiến trình đã đóng job (`KILL_ON_JOB_CLOSE`) | Không thêm chờ; đóng cả các job trong `stopping` |
| m13 | 5 / L8 | Chỉ liệt kê `embed.rs:146`, `main.rs:519`; đường dẫn profile chứa tên người dùng Windows vẫn được log (`session.rs:168`) | Ghi `id` thay cho đường dẫn đầy đủ |
| m14 | plan.md | Phase 1 đã commit (`87bad24`) nhưng bảng vẫn ghi "đang làm" | Cập nhật trạng thái |

## Bổ sung kiểm thử tay

1. Bấm "Khởi động lại" tài khoản đang treo, rồi xóa ngay tài khoản đó: không còn thư mục profile, không còn tiến trình Zalo nào cho id đó (B1).
2. Treo tài khoản chưa chọn rồi bấm chọn nó; thử ở cả hai chế độ; ở chế độ gắn kèm thêm thao tác kéo đổi kích thước (M-2).
3. Gửi tin 1–2 giây sau khi Alt+Tab rời MultiZ (M-6).
4. Chế độ gắn kèm, MultiZ ở khay, gửi tin tới tài khoản đang chọn; khởi động bằng `--tray` ở chế độ gắn kèm (M-7).
5. Khóa `config.json` độc quyền rồi khởi động; dùng file ANSI (M-8).
6. Icon khay và icon toast ở DPI 150% (M-3).
7. Log có dòng xác nhận timeout UIA đã áp dụng, kèm thời gian đọc p50/p95 (M-4, M-5).
8. Mục 5 hiện tại ("phiên mới mở sau khi phiên cũ thoát hẳn") không quan sát được bằng mắt: cần dòng log "phiên cũ thoát sau X ms, mở phiên mới".

## Giả định đã kiểm, không phản đối

- `WM_SHOWWINDOW` (M4): được gửi khi trạng thái hiển thị đổi, kể cả khi `ShowWindow` gọi từ tiến trình khác. Bản thứ hai gọi `SW_SHOW` trước (`main.rs:1029`), không rơi vào ngoại lệ `SW_SHOWNORMAL`. Lời gọi lồng trong `show_main` không gây hại vì cờ đã được xóa trước (`main.rs:469`).
- `ShowWindowAsync` với cửa sổ đã `SetParent`: chỉ đưa yêu cầu vào hàng đợi của luồng sở hữu, không phụ thuộc quan hệ cha con.
- L9: bỏ phần chặn `VK_RETURN` và để `IsDialogMessageW` xử lý là đúng: nút Hủy có id 2, OK có id 1 (`dialog.rs:19-20, 146-151`).
- windows 0.58 có `IUIAutomation2` trong feature `Win32_UI_Accessibility` đã bật; vấn đề nằm ở CLSID (M-4).
