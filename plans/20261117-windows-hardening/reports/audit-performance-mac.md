# Audit hiệu năng, trải nghiệm và bản macOS

Ngày: 2026-11-17. Phạm vi: `windows/src/*.rs`, `mac/Sources/*.swift`, `mac/build.sh`, `mac/Resources/Info.plist`. Phương pháp: đọc mã, không chạy; bản Mac không build được trên máy này. Không lặp lại các mục đã biết (thiếu test, thiếu `.gitignore`, cảnh báo clippy, watcher phụ thuộc bố cục).

Độ tin cậy: **confirmed** = suy ra trực tiếp từ mã hoặc tài liệu API; **plausible** = có kịch bản cụ thể nhưng chưa đo hoặc chưa tái hiện.

## Tóm tắt

| # | Mức | Tin cậy | Vấn đề |
|---|-----|---------|--------|
| W1 | high | confirmed | Mỗi thông báo rò một HICON |
| W2 | high | plausible | "Tắt/Khởi động lại" một Zalo bị treo làm MultiZ treo theo |
| W3 | high | confirmed/plausible | GDI+ khóa `avatar.png`: đổi ảnh lần hai lỗi, xóa tài khoản để lại dữ liệu |
| W4 | medium | plausible | Watcher UIA quét toàn cây, chi phí CPU chủ yếu nằm trong các tiến trình Zalo |
| W5 | medium | plausible | Hàng đợi nhập liệu dùng chung khi gắn cửa sổ chéo tiến trình; `stop` ngủ tới 3 s trên luồng UI |
| W6 | medium | plausible | Kéo hoặc đổi kích thước làm N cửa sổ Zalo dựng lại bố cục ở mỗi thông điệp |
| W7 | medium | confirmed | Bấm thông báo mở tài khoản được báo gần nhất, không phải tài khoản trên thông báo |
| W8 | low | plausible | Ngưỡng 3 s của HSHELL_FLASH thấp hơn chu kỳ thực của watcher, gây thông báo chung và đếm sai |
| W9 | low | plausible | Tắt Zalo bằng `TerminateJobObject` (kill cứng) |
| W10 | low | confirmed | Các điểm UX nhỏ: không có phím tắt, không nhớ vị trí cửa sổ, cuộn bỏ qua delta, Enter trên nút "Hủy" lại chạy OK, lỗi SetParent không báo người dùng |
| W11 | low | confirmed | Tick, vẽ lại và ghi log: chi phí không đáng kể, có vài cơ hội tối ưu nhỏ |
| M1 | medium | plausible | `restore()` nhận nhầm PID đã bị tái sử dụng |
| M2 | medium | plausible | Theo dõi PID dựa vào notification; nếu lỡ thì `stop()` có thể gửi SIGTERM tới tiến trình lạ |
| M3 | medium | plausible | Xóa tài khoản chuyển profile vào Thùng rác sau 1,5 s cố định khi Zalo có thể còn chạy |
| M4 | medium | plausible | LaunchAgent ghi đường dẫn App Translocation; thiếu `AbandonProcessGroup` |
| M5 | low | confirmed | "Tắt Zalo và thoát" thoát sau 1 s nên bước buộc tắt sau 6 s không bao giờ chạy |
| M6 | low | confirmed | Mở lại mọi tài khoản cùng lúc, không giãn cách |
| M7 | low | plausible | Symlink Keychain dùng đường dẫn tuyệt đối, không sửa khi bị gãy; các tài khoản dùng chung Keychain |
| M8 | low | plausible | Không chặn chạy hai bản MultiZ |
| M9 | low | confirmed | Ghi log, sao lưu config hỏng, `build.sh` nuốt lỗi, quyền Automation bị bỏ qua im lặng |

---

## Phần A. Windows

### W1. Mỗi thông báo rò một HICON (high, confirmed)
- `windows/src/tray.rs:72-74`: `notify` gọi `LoadImageW` (không có `LR_SHARED`) để lấy `hBalloonIcon` và không bao giờ `DestroyIcon`. `tray.rs:32-39` (`add`) cũng rò, nhưng chỉ khi Explorer khởi động lại.
- Tác động: mỗi thông báo mất một USER object. Giới hạn mặc định của một tiến trình là 10.000. Một máy bán hàng chạy liên tục dưới khay và nhận vài trăm thông báo mỗi ngày sẽ chạm trần sau vài tuần. Khi đó menu, tooltip, hộp thoại và icon không tạo được nữa.
- Đề xuất: nạp icon một lần lúc khởi động, giữ trong `App` và dùng lại; hoặc gọi `DestroyIcon` ngay sau `Shell_NotifyIconW`, vì shell đã sao chép icon.

### W2. Thao tác trên một Zalo bị treo làm MultiZ treo theo (high, plausible)
- `main.rs:303-308`: `stop` gọi `ShowWindow(s.hwnd, SW_HIDE)` **trước** `session::stop` → `TerminateJobObject`. Với cửa sổ của luồng khác, `ShowWindow` chờ đồng bộ. MSDN khuyên dùng `ShowWindowAsync` chính là để tránh treo khi luồng đích không phản hồi.
- Các lời gọi chéo tiến trình khác có cùng rủi ro: `embed.rs:76,83,108` (`SetWindowPos`), `embed.rs:110` và `main.rs:529` (`ShowWindow`), `embed.rs:19,43` (`SetParent`).
- Kịch bản: Zalo đứng hình, người dùng bấm "Khởi động lại" hoặc "Tắt" để cứu. MultiZ kẹt ở `ShowWindow` và không bao giờ tới bước kill. Kết quả là cả MultiZ và mọi tài khoản khác đều treo. Thao tác dùng để chữa lỗi lại chính là thao tác gây treo.
- Đề xuất: trong `stop`, gọi `TerminateJobObject` trước và bỏ `ShowWindow`, vì cửa sổ sẽ bị hủy cùng tiến trình. Ở `place`, `park` và `tick`, dùng `SWP_ASYNCWINDOWPOS` hoặc `ShowWindowAsync`, hoặc kiểm tra `IsHungAppWindow` trước khi gọi.

### W3. GDI+ khóa file `avatar.png` (high; khóa file: confirmed, mức dữ liệu còn sót: plausible)
- `gfx.rs:183`: `GdipCreateBitmapFromFile` giữ file mở tới khi `GdipDisposeImage` chạy (đây là hành vi đã ghi trong tài liệu của GDI+). `App.avatars` giữ ảnh suốt vòng đời ứng dụng.
- Hệ quả:
  1. Đổi ảnh lần hai: `avatar.rs:127` gọi `rename(tmp → avatar.png)` và lỗi chia sẻ file (os error 32). Người dùng thấy hộp thoại lỗi thô do `actions.rs:74` hiện.
  2. "Dùng lại chữ viết tắt": `actions.rs:360` xóa file **trước** khi bỏ ảnh khỏi map ở `actions.rs:362`. Lệnh xóa lỗi nhưng bị bỏ qua im lặng (`avatar.rs:43`), nên lần mở sau ảnh cũ hiện lại.
  3. Xóa tài khoản: `actions.rs:91` gọi `remove_dir_all` **trước** `a.avatars.remove` ở `actions.rs:94`. Lệnh xóa lỗi và chỉ được ghi log. Ít nhất `avatar.png` và thư mục profile còn lại. Tùy thứ tự duyệt, các thư mục `Documents`, `Downloads`, `Pictures` (nơi Zalo lưu file nhận được) có thể cũng còn. Điều này trái với câu hứa "dữ liệu … sẽ bị xóa" trong hộp thoại xác nhận.
- Đề xuất: đọc file vào bộ nhớ rồi tạo bitmap từ stream (`SHCreateMemStream` + `GdipCreateBitmapFromStream`), hoặc clone bitmap rồi giải phóng bản gốc. Ngoài ra nên đổi thứ tự: bỏ ảnh khỏi `avatars` trước khi xóa file hoặc thư mục, và báo cho người dùng khi xóa không hết.

### W4. Chi phí CPU của watcher UI Automation (medium, plausible)
- `watcher.rs:87-95`: mỗi 2,5 s, với **mỗi** tài khoản, watcher gọi `FindAllBuildCache(TreeScope_Descendants, ControlType=Text)` trên gốc cửa sổ Zalo. Lời gọi này duyệt toàn bộ cây trợ năng, gồm cả khung tin nhắn bên phải (thường là phần lớn nút), rồi bỏ đi hầu hết vì bộ lọc vùng ở `watcher.rs:113`.
- Chi phí nằm chủ yếu ở các tiến trình Zalo chứ không ở MultiZ:
  - Chromium tự bật chế độ trợ năng khi có client UIA truy vấn. Renderer phải duy trì cây trợ năng cho mọi thay đổi DOM, kể cả giữa hai lần quét, ở tất cả các phiên.
  - Mỗi `FindAll` buộc tiến trình browser của Zalo tính lại bounding rect cho mọi nút.
  - Chi phí tăng theo số tài khoản và độ dài hội thoại đang mở. Chu kỳ thực của watcher là 2,5 s cộng tổng thời gian đọc của mọi tài khoản (`watcher.rs:146-158`). Các lần đọc chạy tuần tự và không có timeout, nên một Zalo phản hồi chậm làm chậm việc phát hiện tin cho mọi tài khoản. Transaction timeout mặc định của UIA khoảng 20 s.
- Phía MultiZ còn lãng phí nhỏ: `CreatePropertyCondition` và `CreateCacheRequest` được tạo lại mỗi lần đọc (`watcher.rs:87-90`). Tài khoản `active` vẫn được quét đầy đủ dù kết quả chỉ dùng để đặt lại `prev` (`watcher.rs:184`).
- Tối ưu giữ nguyên hành vi, có thể đo được:
  1. Tạo condition và cache request một lần cho cả luồng.
  2. Thu hẹp phạm vi: tìm phần tử chứa danh sách hội thoại (bounding rect giao vùng x∈[56, x_max]) bằng `TreeWalker` có cắt nhánh theo bounding rect, lưu lại theo hwnd, chỉ `FindAll` dưới phần tử đó, và tìm lại khi lỗi. Cách này bỏ được khung tin nhắn, phần chiếm nhiều nút nhất.
  3. Không quét khi `active`. Khi tài khoản vừa rời trạng thái active, đặt `seen_once = false` để lần quét đầu tiên chỉ lấy mốc. Hành vi tương đương hiện tại, vì tin đến trong lúc đang xem vẫn bị nuốt như bây giờ.
  4. Dùng `IUIAutomation2::put_TransactionTimeout` và `put_ConnectionTimeout` (khoảng 1–2 s) để giới hạn thời gian chờ một Zalo bị treo.
- Cách đo: ghi log `arr.Length()`, số dòng giữ lại và thời gian đọc (p50/p95) sau mỗi khoảng 100 chu kỳ. Hiện chỉ log ở lần đọc đầu tiên (`watcher.rs:174-181`). Thêm vào đó, so CPU tổng của các tiến trình Zalo trong Process Explorer khi bật và khi tắt watcher, với 3–5 tài khoản.

### W5. Hàng đợi nhập liệu dùng chung; `stop` ngủ trên luồng UI (medium, plausible)
- `embed.rs:19` (`SetParent` chéo tiến trình) và `embed.rs:28` (owner chéo tiến trình) khiến Windows gắn hàng đợi nhập liệu của luồng UI MultiZ với luồng UI của mọi Zalo đã gắn.
- `session.rs:189-194`: `stop` chặn luồng UI bằng `sleep` tới 3 s và không bơm thông điệp. Hàm này được gọi khi Tắt, Khởi động lại và Xóa tài khoản.
- Tác động: trong thời gian chờ, bàn phím và chuột của **mọi** Zalo đã nhúng cũng đứng. Theo chiều ngược lại, một Zalo bận lâu trên luồng UI làm chậm nhập liệu của MultiZ và các tài khoản khác.
- Đề xuất: không chờ đồng bộ. Sau `TerminateJobObject`, chuyển việc đợi job rỗng sang `tick`, hoặc dùng `RegisterWaitForSingleObject` trên job handle rồi `PostMessage`. Chỉ `start` lại khi job đã rỗng.

### W6. Kéo hoặc đổi kích thước làm mọi phiên dựng lại bố cục (medium, plausible)
- `main.rs:738-748`: `WM_SIZE` gọi `layout()` → `place_all()`. Ở chế độ nhúng, `park` (`embed.rs:104-108`) đổi kích thước **mọi** cửa sổ đang cất theo bề rộng pane. Ở chế độ gắn kèm, `WM_MOVE | WM_WINDOWPOSCHANGED` gọi `place_all` và mỗi lần gọi `ShowWindow(SW_HIDE)` cho mọi cửa sổ không chọn. Vì `DefWindowProc` của `WM_WINDOWPOSCHANGED` sinh thêm `WM_MOVE` và `WM_SIZE`, `place_all` chạy hai lần cho mỗi bước.
- Tác động: khi kéo mép cửa sổ, mỗi bước chuột gây ra N lời gọi `SetWindowPos` đồng bộ chéo tiến trình, và N renderer Chromium phải tính lại bố cục. Với 5 tài khoản trở lên, CPU tăng vọt và thao tác kéo bị giật.
- Đề xuất: trong khoảng `WM_ENTERSIZEMOVE` đến `WM_EXITSIZEMOVE`, chỉ đặt lại cửa sổ đang chọn; cửa sổ cất đi thì đổi kích thước một lần khi kết thúc. Ở `park`, bỏ qua `SetWindowPos` khi kích thước không đổi. Bỏ `WM_MOVE` khỏi nhánh xử lý vì `WM_WINDOWPOSCHANGED` đã bao gồm nó.

### W7. Bấm thông báo mở sai tài khoản (medium, confirmed)
- `main.rs:438` và `main.rs:890-897`: `NIN_BALLOONUSERCLICK` dùng `last_notified_id`, tức tài khoản được báo **gần nhất**.
- Kịch bản: A báo trước, B báo sau. Người dùng bấm thông báo của A trong Action Center và MultiZ chuyển sang B.
- Đề xuất: nếu vẫn dùng balloon, chỉ có thể giữ cách hiện tại. Cách giảm thiểu là đặt tên tài khoản ở đầu tiêu đề (đã có) và chọn tài khoản có `has_new` cũ nhất khi `last_notified_id` không còn `has_new`. Giải pháp đúng là chuyển sang toast có tham số kích hoạt, nhưng việc đó thay đổi phạm vi.

### W8. Ngưỡng 3 s của HSHELL_FLASH và chu kỳ watcher (low, plausible)
- `main.rs:601-611` báo "Có tin nhắn mới" nếu watcher không bắt được tin trong 3 s. Chu kỳ thực của watcher là 2,5 s cộng thời gian đọc mọi tài khoản (xem W4), dễ vượt 3 s khi có từ 3–5 tài khoản.
- Tác động: watcher bắt được tin muộn, sau thông báo chung, nên `new_message` rơi vào nhánh cooldown (`main.rs:410-419`) và 8 s sau gửi thêm "2 tin nhắn mới" cho một tin duy nhất.
- Đề xuất: đặt ngưỡng flash theo chu kỳ watcher đo được (ví dụ chu kỳ gần nhất cộng 1 s), hoặc không tăng `count` khi thông báo trước đó là thông báo chung từ flash.

### W9. Tắt Zalo bằng kill cứng (low, plausible)
- `session.rs:187` và `actions.rs:430`: các thao tác Tắt, Khởi động lại và Thoát đều gọi `TerminateJobObject`. Bản Mac (`ZaloLauncher.swift:115-125`) thoát êm trước, chỉ buộc tắt sau 6 s.
- Tác động: dữ liệu cục bộ chưa ghi xuống (bản nháp, trạng thái đồng bộ, CSDL tin) có thể mất hoặc phải khôi phục khi mở lại.
- Đề xuất: gửi `WM_QUERYENDSESSION`/`WM_ENDSESSION` (Electron xử lý như lệnh thoát) tới cửa sổ chính của phiên, chờ không đồng bộ khoảng 3–5 s, rồi mới kill job.

### W10. UX: các điểm yếu nhỏ đã xác nhận (low, confirmed)
- Không có phím tắt: không tìm thấy `RegisterHotKey` hay bảng accelerator. Vì focus nằm trong Zalo, một luồng khác, MultiZ không nhận được phím. Đề xuất: đăng ký `Ctrl+1..9` và `Ctrl+Tab` bằng `RegisterHotKey` khi MultiZ được kích hoạt và hủy khi mất kích hoạt (`WM_ACTIVATE`, `main.rs:858`).
- Không nhớ vị trí và kích thước cửa sổ: `main.rs:1063-1083` luôn đặt cửa sổ 1280×820 ở giữa màn hình chính, và `Config` không có trường nào cho việc này.
- Cuộn chuột bỏ qua độ lớn của delta (`main.rs:813-817`): touchpad chính xác gửi nhiều delta nhỏ nên danh sách nhảy quá nhanh. Đề xuất: cộng dồn delta và chỉ cuộn một dòng mỗi 120 đơn vị.
- Hộp thoại tài khoản: `dialog.rs:317-320` bắt mọi `VK_RETURN` trước `IsDialogMessageW`, nên khi focus đang ở nút "Hủy", nhấn Enter lại chạy OK.
- `embed.rs:16-22`: style `WS_CHILD` được đặt **trước** `SetParent` và không được khôi phục khi `SetParent` lỗi. Lỗi chỉ ghi log, pane không có gợi ý gì. Đề xuất: khôi phục style, ghi `failed[id]` với gợi ý chuyển sang chế độ gắn kèm.
- `Config::save` (`config.rs:80-88`) bỏ qua lỗi ghi im lặng. Lỗi `CreateProcessW` chỉ hiện mã số (`session.rs:160`), ví dụ 740 khi Zalo yêu cầu quyền quản trị; nên dịch các mã phổ biến (2, 5, 740) thành câu dễ hiểu.

### W11. Tick, vẽ lại, ghi log: chi phí không đáng kể (low, confirmed)
- Tick 400 ms (`main.rs:480-653`): mỗi phiên tốn một `QueryInformationJobObject` kèm cấp phát và xóa trắng bộ đệm 16 KB (`session.rs:49`), khoảng 5–8 lời gọi user32 và một `String` tiêu đề. Ngoài ra còn `Vec<Target>` clone id, chuỗi `sig` và `app_active` (thêm một lần `pids`). Với 5 tài khoản, mỗi tick chỉ mất cỡ chục micro giây, dưới 0,01 % CPU. `EnumWindows` chỉ chạy khi phiên chưa có cửa sổ. Phiên Zalo chạy ẩn vĩnh viễn dưới khay sẽ quét mỗi 400 ms, nhưng chi phí vẫn nhỏ. Tối ưu tùy chọn: bộ đệm PID 64 phần tử trên stack và chỉ cấp phát lại khi gặp `ERROR_MORE_DATA`; giãn `best_window` ra 2 s sau 20 s đầu.
- Vẽ lại: font được cache (`ui.rs:33-45`); mọi brush, pen, path GDI+ được tạo và hủy trong cùng lần vẽ, không rò. `double_buffer` (`main.rs:684-698`) tạo bitmap bằng cả vùng client (khoảng 4 MB ở 1280×820) mỗi lần vẽ, và đổi hover làm vẽ lại toàn bộ cửa sổ chính. Tần suất thấp vì đã có `sig` chặn vẽ thừa. Tối ưu tùy chọn: giữ lại back buffer giữa các lần vẽ, chỉ invalidate vùng sidebar khi đổi hover, bỏ `accounts.clone()` mỗi lần vẽ (`ui.rs:188,343`).
- Log (`util.rs:49-67`): mỗi dòng mất một `metadata`, một lần mở file và một lần ghi, gọi từ hai luồng; việc đổi tên khi vượt 1 MB có thể đua giữa hai luồng nhưng vô hại. Tần suất thấp và có chống lặp (`logged_empty`, `logged_err`). Log không ghi nội dung tin, chỉ ghi tiêu đề cửa sổ dạng "(3) Zalo".

### Điểm mạnh nên phát huy
- Một file exe nhỏ (`opt-level="s"`, LTO, strip, `panic=abort`), chỉ có ba dependency (`serde`, `serde_json`, `windows`/`windows-sys`) và không có mã mạng: URL chỉ được mở bằng trình duyệt qua `ShellExecuteW`.
- Không sửa và không chèn mã vào Zalo; tách profile bằng biến môi trường; Job Object với `KILL_ON_JOB_CLOSE` bảo đảm không để lại tiến trình mồ côi.
- Lưu config nguyên tử, sao lưu file hỏng; log không chứa nội dung tin; có tùy chọn ẩn nội dung trong thông báo.
- Tự mở tài khoản giãn cách 1,5 s; gom thông báo theo `NOTIFY_COOLDOWN`; chặn vẽ thừa bằng `sig`; nhận biết DPI theo từng màn hình (PMv2) và xử lý `WM_DPICHANGED`; avatar thu nhỏ về 96 px ngay khi nhập.
- Có các thông báo lỗi hướng dẫn hành động ("Thử lại", "Gắn lại cửa sổ") khi Zalo thoát sớm hoặc mất cửa sổ.

---

## Phần B. macOS

### M1. `restore()` nhận nhầm PID đã tái sử dụng (medium, plausible)
- `ZaloLauncher.swift:62-73` chỉ kiểm tra hai điều: PID đang sống và `executableURL` trùng Zalo.app. Không kiểm tra profile hay thời điểm tiến trình khởi động. File pid cũ không bao giờ được dọn khi tiến trình đã chết.
- Kịch bản: máy khởi động lại, Zalo gốc của người dùng là login item và tình cờ nhận đúng PID ghi trong `profiles/<id>/zalo.pid` của lần chạy trước. Trong cùng một lần khởi động hệ thống, một phiên Zalo khác do MultiZ mở cũng có thể nhận PID đó. MultiZ nhận tiến trình đó là tài khoản X: menu hiện X đang chạy, "Chuyển tới" mở nhầm Zalo, "Tắt" tắt Zalo cá nhân của người dùng.
- Đề xuất: ghi kèm thời điểm khởi động tiến trình vào file pid (`proc_pidinfo` với `PROC_PIDTBSDINFO`, trường `pbi_start_tvsec`) và so khớp khi khôi phục, hoặc đọc môi trường tiến trình (`sysctl KERN_PROCARGS2`) để kiểm tra `CFFIXED_USER_HOME` đúng profile. Xóa file pid khi kiểm tra không khớp.

### M2. Theo dõi PID dựa vào notification; `kill` thô (medium, plausible)
- `ZaloLauncher.swift:13-24`: trạng thái chỉ được dọn khi nhận `didTerminateApplicationNotification`. Notification này chỉ có với tiến trình đã đăng ký với LaunchServices. Nếu Zalo thoát rất sớm (hỏng profile, bản App Store bị sandbox chặn), có thể không có notification.
- `isRunning` (`:48-51`) dùng `kill(pid, 0)`. Sau khi PID bị tái sử dụng, hàm này trả về true với một tiến trình bất kỳ của người dùng. Khi đó `stop` (`:115-119`) không tìm thấy `NSRunningApplication` và chuyển sang `kill(pid, SIGTERM)`, tức gửi tín hiệu tới một tiến trình lạ.
- Đề xuất: giữ tham chiếu `Process` và dùng `terminationHandler`, cách này tin cậy hơn vì dựa trên `waitpid`. Bỏ nhánh `kill(pid, SIGTERM)` khi không xác minh được tiến trình là Zalo của profile đó.

### M3. Xóa tài khoản khi Zalo có thể vẫn chạy (medium, plausible)
- `AppDelegate.swift:248-254`: hàm gọi `stop` rồi chờ cố định 1,5 s và `trashItem(profile)`. Trong khi đó `stop` cho Zalo tới 6 s trước khi buộc tắt (`ZaloLauncher.swift:121-125`).
- Tác động: Zalo còn sống tiếp tục ghi vào các file đã bị chuyển vào Thùng rác qua fd đang mở, và tạo lại `profiles/<id>/Library/...` theo đường dẫn HOME. Kết quả là thư mục mồ côi còn trên đĩa sau khi xóa, có thể chứa phiên đăng nhập.
- Đề xuất: chỉ chuyển vào Thùng rác sau khi xác nhận tiến trình đã thoát (notification, `terminationHandler`, hoặc kiểm tra định kỳ có giới hạn thời gian) và sau bước buộc tắt.

### M4. LaunchAgent (medium, plausible)
- `LoginItem.swift:39-44` ghi `Bundle.main.executablePath`. App ký ad-hoc, tải về dạng zip nên mang cờ quarantine. Nếu người dùng chạy ngay trong `~/Downloads`, Gatekeeper chạy app qua App Translocation với đường dẫn ngẫu nhiên `/private/var/folders/.../AppTranslocation/...`. Agent ghi đường dẫn đó, nên sau khi khởi động lại máy app không tự mở, và không có thông báo nào.
- Plist thiếu `AbandonProcessGroup = true`. Khi MultiZ được launchd mở và người dùng chọn "Chỉ thoát MultiZ", launchd sẽ diệt các tiến trình còn lại cùng process group, nếu Zalo con cùng nhóm (tùy cách `Process` đặt pgid). Khi đó hành vi khác với lúc MultiZ được mở từ Finder.
- Đề xuất: phát hiện `/AppTranslocation/` trong đường dẫn, nhắc người dùng chuyển app vào /Applications và không ghi agent. Thêm `AbandonProcessGroup = true`. Trên macOS 13+ có thể dùng `SMAppService.mainApp`.

### M5. "Tắt Zalo và thoát" bỏ bước buộc tắt (low, confirmed)
- `AppDelegate.swift:338-339`: `NSApp.terminate` chạy sau 1 s, nên các closure buộc tắt sau 6 s (`ZaloLauncher.swift:121`) không bao giờ chạy. Zalo nào không thoát trong 1 s sẽ tiếp tục chạy mồ côi, file pid còn nguyên, và gặp lại rủi ro của M1.
- Đề xuất: đợi mọi phiên thoát, có giới hạn khoảng 7 s, rồi mới `terminate`; dùng `applicationShouldTerminate` với `.terminateLater`.

### M6. Mở lại mọi tài khoản cùng lúc (low, confirmed)
- `AppDelegate.swift:42-46` mở mọi tài khoản `reopen` trong một vòng lặp, không giãn cách. Bản Windows giãn 1,5 s (`main.rs:627-634`).
- Tác động: lúc đăng nhập hệ thống, N tiến trình Electron khởi động đồng thời, gây đỉnh CPU và I/O và làm máy chậm hẳn trong vài chục giây.
- Đề xuất: dùng hàng đợi với `asyncAfter` 1,5 s, ưu tiên tài khoản dùng gần nhất.

### M7. Symlink Keychain (low, plausible)
- `ZaloLauncher.swift:164-168`: symlink trỏ tới đường dẫn tuyệt đối `/Users/<tên>/Library/Keychains`. `checkResourceIsReachable` đi theo link nên với link gãy sẽ trả false, rồi `createSymbolicLink` lỗi vì đường dẫn đã tồn tại, và lỗi bị nuốt bởi `try?`. Link gãy vì vậy không bao giờ được sửa.
- Kịch bản: thư mục MultiZ được chép sang tài khoản macOS khác hoặc máy khác có tên người dùng khác. Zalo không truy cập được Keychain, có thể mất đăng nhập hoặc báo lỗi lưu khóa.
- Mọi tài khoản dùng chung login keychain thật. Nếu Zalo lưu bí mật theo một khóa cố định cho từng phiên đăng nhập, tài khoản đăng nhập sau sẽ ghi đè tài khoản trước và tài khoản trước buộc phải đăng nhập lại. Cần thử trên máy thật.
- Đề xuất: dùng `destinationOfSymbolicLink`; nếu đích khác Keychain thật hiện tại thì xóa link và tạo lại. Ghi log khi tạo link lỗi.

### M8. Không chặn chạy hai bản MultiZ (low, plausible)
- Bản Windows có mutex (`main.rs:1023-1034`); bản Mac không có kiểm tra tương tự. Hai bản sao `.app` ở hai đường dẫn khác nhau (bản cũ trong Downloads mà agent trỏ tới, bản mới trong /Applications) có thể chạy song song. Khi đó có hai biểu tượng trên thanh menu, và nếu hai bản khởi động gần như cùng lúc thì có thể mở hai Zalo trên cùng một profile.
- Đề xuất: lúc khởi động, kiểm tra `NSRunningApplication.runningApplications(withBundleIdentifier:)`; nếu đã có bản khác thì kích hoạt bản đó rồi thoát.

### M9. Các lỗi nhỏ (low, confirmed)
- `Store.swift:70`: `copyItem` không ghi đè, nên lần hỏng config thứ hai không sao lưu được. Lần lưu tiếp theo sẽ ghi đè bản gốc duy nhất. Bản Windows dùng `fs::copy`, có ghi đè.
- `Store.swift:130-133`: log bị **xóa** khi vượt 1 MB, trong khi bản Windows giữ `log.old.txt`. `FileHandle.write` (API cũ) ném ngoại lệ ObjC khi đĩa đầy, làm ứng dụng crash. `DateFormatter` được tạo mới mỗi dòng log.
- `ZaloLauncher.swift:155`: lỗi Apple Event bị nuốt. Nếu người dùng từ chối quyền Automation, "chuyển tới" không mở lại được cửa sổ mà không có thông báo nào. Chữ ký ad-hoc đổi sau mỗi bản build nên quyền TCC có thể phải cấp lại sau mỗi lần cập nhật.
- `build.sh:26` ẩn lỗi biên dịch arm64 (`2>/dev/null`) và lặng lẽ đóng gói bản chỉ có Intel; máy Apple silicon chưa cài Rosetta sẽ không mở được. `build.sh:47` dùng `|| true`, nên ký lỗi vẫn ra zip.
- `AppDelegate.swift:73` và `ZaloLauncher.swift:35`: `fullPath(forApplication:)` (API cũ) tra LaunchServices mỗi lần mở menu, làm menu mở chậm.

### Khác biệt hành vi so với bản Windows
- Không có các tính năng: phát hiện tin mới, thông báo, số chưa đọc, nhúng cửa sổ (đã biết).
- Thoát ứng dụng: Windows luôn tắt mọi Zalo; Mac cho chọn "Chỉ thoát MultiZ" và khôi phục lại phiên bằng file pid.
- Tự mở: Windows dùng cờ `autostart` do người dùng bật tắt, mặc định true; Mac đặt `reopen` tự động (thành true khi mở, false khi bấm "Tắt"). Nếu người dùng thoát từ trong Zalo (Cmd+Q), `reopen` vẫn là true.
- Tắt Zalo: Windows kill cứng (W9); Mac thoát êm, buộc tắt sau 6 s.
- Xóa tài khoản: Windows xóa hẳn (nhưng bị W3 cản); Mac chuyển vào Thùng rác (nhưng bị M3 cản).
- Mở lại khi khởi động: Windows giãn cách, Mac mở đồng loạt (M6).
- Kích thước avatar: Windows 96 px, Mac 128 px. Bảng màu khớp nhau.

## Việc nên làm, xếp theo ưu tiên
1. W1: hủy hoặc dùng lại HICON (sửa vài dòng).
2. W2 và W5: kill trước, bỏ `ShowWindow` đồng bộ trong `stop`, đợi job rỗng không đồng bộ.
3. W3: nạp avatar qua bộ nhớ; bỏ ảnh khỏi map trước khi xóa.
4. W4: đo trước (log thời gian và số phần tử), sau đó thu hẹp phạm vi `FindAll`, tạo condition một lần, đặt timeout UIA.
5. M1, M2, M3: xác minh PID bằng thời điểm khởi động; dùng `terminationHandler`; xóa profile sau khi tiến trình đã thoát.
6. W6, M4, M5, M6: tối ưu lúc đổi kích thước; sửa LaunchAgent; thoát và mở lại có giãn cách.

## Câu hỏi còn mở
- Zalo trên Mac có lưu bí mật phiên vào Keychain theo một khóa cố định không? Cần thử hai tài khoản trên máy thật để biết M7 có làm tài khoản đăng xuất lẫn nhau không.
- `Process` trên Darwin có đặt process group mới cho tiến trình con không? Câu trả lời quyết định phần `AbandonProcessGroup` của M4 có ảnh hưởng thực tế hay không.
- Chi phí thực của W4 cần đo với 3–5 tài khoản trên máy của người dùng cuối.
