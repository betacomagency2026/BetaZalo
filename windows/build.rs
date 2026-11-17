use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=res/app.rc");
    println!("cargo:rerun-if-changed=res/app.manifest");
    println!("cargo:rerun-if-changed=res/app.ico");
    println!("cargo:rerun-if-env-changed=RC");
    println!("cargo:rerun-if-env-changed=WINDRES");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let obj = if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        compile_rc(&out)
    } else {
        compile_windres(&out)
    };
    println!("cargo:rustc-link-arg-bins={}", obj.display());
}

fn compile_windres(out: &std::path::Path) -> PathBuf {
    let obj = out.join("app_res.o");
    let windres = std::env::var("WINDRES").unwrap_or_else(|_| "x86_64-w64-mingw32-windres".into());
    let st = Command::new(&windres)
        .args(["--input-format=rc", "-O", "coff", "-c", "65001", "-i", "res/app.rc", "-o"])
        .arg(&obj)
        .status()
        .unwrap_or_else(|e| panic!("không chạy được {windres}: {e}; cài mingw-w64 hoặc đặt biến WINDRES"));
    assert!(st.success(), "{windres} biên dịch res/app.rc thất bại");
    obj
}

fn compile_rc(out: &std::path::Path) -> PathBuf {
    let res = out.join("app.res");
    let rc = find_rc();
    let st = Command::new(&rc)
        .current_dir("res")
        .args(["/nologo", "/c65001", "/fo"])
        .arg(&res)
        .arg("app.rc")
        .status()
        .unwrap_or_else(|e| panic!("không chạy được {}: {e}; cài Windows SDK hoặc đặt biến RC", rc.display()));
    assert!(st.success(), "{} biên dịch res/app.rc thất bại", rc.display());
    res
}

fn find_rc() -> PathBuf {
    if let Some(p) = std::env::var_os("RC") {
        return PathBuf::from(p);
    }
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => "x64",
    };
    let roots = [std::env::var_os("ProgramFiles(x86)"), std::env::var_os("ProgramFiles")];
    for root in roots.into_iter().flatten() {
        let bin = PathBuf::from(root).join("Windows Kits").join("10").join("bin");
        let Ok(entries) = std::fs::read_dir(&bin) else { continue };
        let mut versions: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("10.")))
            .collect();
        versions.sort_by_key(|p| version_key(p));
        for v in versions.iter().rev() {
            for host in ["x64", arch, "x86"] {
                let cand = v.join(host).join("rc.exe");
                if cand.is_file() {
                    return cand;
                }
            }
        }
    }
    PathBuf::from("rc.exe")
}

fn version_key(p: &std::path::Path) -> Vec<u32> {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.split('.').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_default()
}
