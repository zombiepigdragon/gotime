# gotime

*¿Por qué no los dos?*

gotime is a Rust asynchronous executor and runtime (like [Tokio](https://docs.rs/tokio/latest/tokio/)) written with Go.
It uses the Go scheduler to run async Rust programs, unifying the best programming language for async with the best programming language for everything but async.

## Platform Support

This only works on Linux, probably- the build script that makes Cargo link to the runtime hardcodes Linux filenames, and it's not been tested anywhere else.
However, with a few changes, it'll likely work on Windows or MacOS.

## Why?

This project isn't serious- you shouldn't do this.
I just thought it would be a funny thing to do, and so I made it work.

You should really use [Tokio](https://docs.rs/tokio/latest/tokio/) or something instead.

## Future Plans

The runtime isn't very useful right now, so I plan to continue expanding it to have more async essentials, from a `#[main]` attribute macro to Go-based IO support.
A distant hope of mine is to make this crate `#![no_std]` in Rust, so that it can use only the (more featureful!) Go standard library.