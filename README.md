# gotime; broken branch

Hi! If you're here because my code doesn't work right now, this is the right branch.

It *probably* won't build on a non-Linux system (mostly untested, see `build.rs`).
You can use `run_tests.sh` for the closest thing to the testing setup I use (cargo-careful is required).
Note that to build you'll either need to enable ASAN (the script does) or comment out the line enabling it for the Go code in `build.rs`.
Please excuse my commented code and/or print debugging attempts...

The issue is in the `main.rs` testbed- the future is being created with `has_run: false` but it magically becomes `true` by the time of first poll.

## Versions

```
❯ go version
go version go1.22.2 linux/amd64
❯ cargo -V
cargo 1.79.0-nightly (c93926759 2024-04-23)
```