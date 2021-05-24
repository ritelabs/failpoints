# failpoints, another fail-rsS

[![CI](https://github.com/ritelabs/failpoints/workflows/CI/badge.svg)](https://github.com/ritelabs/failpoints/actions)
[![Crates.io](https://img.shields.io/crates/v/failpoints.svg?maxAge=2592000)](https://crates.io/crates/failpoints)

[Documentation](https://docs.rs/failpoints).

A failpoints implementation for Rust.

Fail points are code instrumentations that allow errors and other behavior to be injected dynamically at runtime, primarily for testing purposes. Fail points are flexible and can be configured to exhibit a variety of behavior, including panics, early returns, and sleeping. They can be controlled both programmatically and via the environment, and can be triggered conditionally and probabilistically.

This crate is inspired by FreeBSD's [failpoints](https://freebsd.org/cgi/man.cgi?query=fail).

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
failpoints = "0.1"
```

Now you can import the `failpoint!` macro from the `failpoints` crate and use it to inject dynamic failures.
Fail points generation by this macro is disabled by default, and can be enabled where relevant with the `failpoints` Cargo feature.

As an example, here's a simple program that uses a fail point to simulate an I/O panic:

```rust
use failpoints::{failpoint, FailScenario};

fn do_fallible_work() {
    failpoint!("read-dir");
    let _dir: Vec<_> = std::fs::read_dir(".").unwrap().collect();
    // ... do some work on the directory ...
}

fn main() {
    let scenario = FailScenario::setup();
    do_fallible_work();
    scenario.teardown();
    println!("done");
}
```

Here, the program calls `unwrap` on the result of `read_dir`, a function that returns a `Result`. In other words, this particular program expects this call to `read_dir` to always succeed. And in practice it almost always will, which makes the behavior of this program when `read_dir` fails difficult to test. By instrumenting the program with a fail point we can pretend that `read_dir` failed, causing the subsequent `unwrap` to panic, and allowing us to observe the program's behavior under failure conditions.

When the program is run normally it just prints "done":

```sh
$ cargo run
     Finished dev [unoptimized + debuginfo] target(s) in 1.18s
     Running `target/debug/failpoint`
done
```

But now, by setting the `FAILPOINTS` variable we can see what happens if the `read_dir` fails:

```
$ FAILPOINTS=read-dir=panic cargo run --features failpoints/failpoints
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/failpoint`
thread 'main' panicked at 'failpoint read-dir panic', /home/psiace/Projects/ritelabs/dev/failpoints/src/lib.rs:498:25
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

For further information see the [API documentation](https://docs.rs/failpoints).

