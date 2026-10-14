fn main() {
    println!("cargo:rerun-if-changed=assets/windows/app.rc");
    println!("cargo:rerun-if-changed=assets/app.ico");

    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=KHASLANA_TARGET={target}");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            // Kit 的复杂渲染在调试版有较大的按值临时量；默认 1 MiB 主线程栈会溢出。
            // 只调整 GUI 二进制的预留量，保持系统默认提交量和其它目标的链接设置。
            println!("cargo:rustc-link-arg-bin=khaslana=/STACK:8388608");
        }
        embed_resource::compile("assets/windows/app.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed Windows application icon");
    }
}
