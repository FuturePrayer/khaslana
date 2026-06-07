fn main() {
    println!("cargo:rerun-if-changed=assets/windows/app.rc");
    println!("cargo:rerun-if-changed=assets/app.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/windows/app.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed Windows application icon");
    }
}
