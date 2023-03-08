fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let runtime_src = std::fs::canonicalize("src/runtime.go").unwrap();

    assert_eq!(
        std::process::Command::new("go")
            .arg("vet")
            .arg("-mod=vendor")
            .arg("runtime")
            .current_dir(runtime_src.parent().unwrap())
            .status()
            .unwrap()
            .code(),
        Some(0)
    );
    assert_eq!(
        std::process::Command::new("go")
            .arg("build")
            .arg("-buildmode=c-archive")
            .arg(runtime_src)
            .current_dir(&out_dir)
            .status()
            .unwrap()
            .code(),
        Some(0)
    );

    std::fs::rename(out_dir.join("runtime.a"), out_dir.join("libruntime.a")).unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=runtime")
}
