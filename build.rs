fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "1.0.0".to_string());
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/app.ico")
        .set("FileDescription", "Simple Auto Clicker")
        .set("ProductName", "Simple Auto Clicker")
        .set("OriginalFilename", "SimpleAutoClicker.exe")
        .set("FileVersion", &version)
        .set("ProductVersion", &version)
        .set("LegalCopyright", "MIT License");

    if let Err(error) = res.compile() {
        println!("cargo:warning=could not attach the Windows resources: {error}");
    }
}
