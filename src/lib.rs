#[no_mangle]
pub extern "C" fn rust_hello_world() {
    println!("Hello from Rust lib")
}

pub fn rust_hello() {
    println!("Hello from Rust main ");
    unsafe { HelloGo() };
}

// #[link(name = "runtime")]
extern "C" {
    fn HelloGo();
}
