fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/atk_control.ico");
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let rc_path = format!("{}/app.rc", out_dir);
        let ico_path = match std::path::Path::new("assets/atk_control.ico").canonicalize() {
            Ok(p) => p,
            Err(_) => return,
        };
        let ico_str = ico_path.to_str().unwrap().replace('\\', "/");
        if std::fs::write(&rc_path, format!("1 ICON \"{}\"\n", ico_str)).is_err() {
            return;
        }
        let res_path = format!("{}/app.o", out_dir);
        if std::process::Command::new("windres")
            .args(&["-i", &rc_path, "-o", &res_path])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            println!("cargo:rustc-link-arg={}", res_path);
        }
    }
}

