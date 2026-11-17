# Phase 1: Build và lint

## Yêu cầu

- Build được bằng toolchain MSVC mà không cần mingw.
- `cargo clippy --release` không cảnh báo.
- Không theo dõi sản phẩm build trong git.

## File

- `windows/build.rs`: target MSVC gọi `rc.exe` (biến `RC`, sau đó Windows SDK mới nhất), target GNU giữ `windres`.
- `.gitignore`: `windows/target/`, `mac/build/`.
- `windows/src/tray.rs`, `windows/src/main.rs`: thay `1 as *const u16` bằng hàm `make_int_resource`.
- `windows/src/session.rs:119`: `env.values()`.
- `windows/src/util.rs:21`, `windows/src/main.rs:460`: lint nhỏ.
- `windows/src/gfx.rs:85`, `windows/src/ui.rs:54`: `#[allow(clippy::too_many_arguments)]` vì đây là hàm vẽ nhận tọa độ; gom struct không giảm độ phức tạp.

## Kiểm chứng

- Exe có VersionInfo 0.3.4, manifest PerMonitorV2, icon.
- clippy 0 cảnh báo, fmt đạt.

## Rủi ro

- `rc.exe` khác `windres` ở cú pháp: `app.rc` hiện tương thích cả hai (đã thử).
