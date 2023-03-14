fn main() {
    // get some important directory and file paths
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let runtime_src = std::fs::canonicalize("src/runtime.go").unwrap();

    // check that the Go code is correct
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
    // build the Go code
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

    // Make the build result of Go visible to the linker
    // FIXME: This is probably Unix specific, but I couldn't get the right println to link an object file like this.
    std::fs::rename(out_dir.join("runtime.a"), out_dir.join("libruntime.a")).unwrap();

    // use bindgen to import the Cgo definitions
    let bindgen = bindgen::builder()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("gotime_.*")
        .blocklist_function("gotime_poll_task") // exported from Rust not Go
        .use_core()
        .header(out_dir.join("runtime.h").to_str().unwrap());
    let bindings = bindgen.generate().unwrap();
    bindings.emit_warnings();
    bindings.write_to_file(out_dir.join("runtime.rs")).unwrap();

    // Tell Cargo when to rebuild
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/runtime.go");

    // Tell Cargo what we've done
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=runtime")
}
