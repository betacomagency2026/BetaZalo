use std::process::Command;
fn main() {
    println!("cargo:rerun-if-changed=res/app.rc");
    println!("cargo:rerun-if-changed=res/app.manifest");
    println!("cargo:rerun-if-changed=res/app.ico");
    let out = std::env::var("OUT_DIR").unwrap();
    let obj = format!("{}/app_res.o", out);
    let windres = std::env::var("WINDRES").unwrap_or_else(|_| "x86_64-w64-mingw32-windres".into());
    let st = Command::new(windres)
        .args(["--input-format=rc", "-O", "coff", "-c", "65001", "-i", "res/app.rc", "-o", &obj])
        .status()
        .expect("windres");
    assert!(st.success());
    println!("cargo:rustc-link-arg-bins={}", obj);
}
