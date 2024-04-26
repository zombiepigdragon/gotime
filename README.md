# gotime; broken branch

Hi! If you're here because my code doesn't work right now, this is the right branch.

It *probably* won't build on a non-Linux system (mostly untested, see `build.rs`).
You can use `run_tests.sh` for the closest thing to the testing setup I use (cargo-careful is required).
Note that to build you'll either need to enable ASAN (the script does) or comment out the line enabling it for the Go code in `build.rs`.
Please excuse my commented code and/or print debugging attempts...

The issue is in the `main.rs` testbed- the future is being created with `has_run: false` but it magically becomes `true` by the time of first poll.

## Print-Debug Output

With Address Sanitizer:

```
❯ RUSTFLAGS="-Zsanitizer=address -A warnings" cargo r --quiet
fut: before block_on (should be false)
[src/main.rs:51:5] &fut = MyUselessFuture {
    has_run: false,
}
go: handle is 1
go: spawned task
start poll callback
get raw handle
[src/ffi.rs:35:9] raw_handle = 1
create handle
new waker
[src/ffi.rs:38:9] &waker = Waker {
    data: 0x0000000000000001,
    vtable: 0x000055c36438f800,
}
create context
transmute fut
poll fut
fut: start of poll (should be false)
[src/main.rs:35:13] &self = MyUselessFuture {
    has_run: true,
}
fut: post assert (should be false or assert failed)
[src/main.rs:38:13] &self = MyUselessFuture {
    has_run: true,
}
fut: about to do assignment
fut: post assignment (should be true)
[src/main.rs:43:13] &this = MyUselessFuture {
    has_run: true,
}
fut: ready
ready
waker drop
go: finished task
```

Notice that the message "should be false or assert failed" is surrounded by `true`.

Alternatively, without Address Sanitizer:

```
❯ RUSTFLAGS="-A warnings" cargo r --quiet
fut: before block_on (should be false)
[src/main.rs:51:5] &fut = MyUselessFuture {
    has_run: false,
}
go: handle is 1
go: spawned task
start poll callback
get raw handle
[src/ffi.rs:35:9] raw_handle = 1
create handle
new waker
[src/ffi.rs:38:9] &waker = Waker {
    data: 0x0000000000000001,
    vtable: 0x00005af797358458,
}
create context
transmute fut
poll fut
fut: start of poll (should be false)
[src/main.rs:35:13] &self = MyUselessFuture {
    has_run: true,
}
thread '<unnamed>' panicked at src/main.rs:36:13:
assertion failed: !self.has_run
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
waker drop
panic!
panic msg: assertion failed: !self.has_run
zsh: IOT instruction (core dumped)  RUSTFLAGS="-A warnings" cargo r --quiet
```

Note that `fut.has_run` somehow becomes true at the start of `poll`.

## Versions

```
❯ go version
go version go1.22.2 linux/amd64
❯ cargo -V
cargo 1.79.0-nightly (c93926759 2024-04-23)
```