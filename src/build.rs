use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/icons/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Copy icon vào OUT_DIR, chỉ dùng tên file (không có khoảng trắng) khi gọi windres
    if let Err(e) = std::fs::copy("assets/icons/icon.ico", out.join("app.ico")) {
        println!("cargo:warning=không copy được icon: {e}");
        return;
    }
    if std::fs::write(out.join("app.rc"), "1 ICON \"app.ico\"\r\n").is_err() {
        println!("cargo:warning=không ghi được app.rc");
        return;
    }

    // Chạy windres trong OUT_DIR với tên file ngắn, không có khoảng trắng
    let status = Command::new("windres")
        .current_dir(&out)
        .args(["app.rc", "-O", "coff", "-o", "app.res"])
        .status();
    match status {
        Ok(s) if s.success() => {
            let res = out.join("app.res");
            println!("cargo:rustc-link-arg-bins={}", res.display());
        }
        Ok(s) => println!("cargo:warning=windres lỗi (mã {s}), exe không có icon"),
        Err(e) => println!("cargo:warning=không chạy được windres ({e}), exe không có icon"),
    }
}