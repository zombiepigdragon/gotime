//! Imports and exports to the Gotime runtime.

extern "C" {
    pub fn HelloGo();
}

#[no_mangle]
pub extern "C" fn rust_hello_world() {
    println!("Hello from Rust lib")
}
