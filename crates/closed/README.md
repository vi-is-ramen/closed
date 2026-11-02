# `closed`

[![Crates.io](https://img.shields.io/crates/v/closed.svg)](https://crates.io/crates/closed)
[![Documentation](https://docs.rs/closed/badge.svg)](https://docs.rs/closed)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> Type-erased callable objects for Rust. Modeled after C++'s `std::function`,
> with HRTB, async, and `no_std` support.

Storing a closure in a struct field is painful in Rust. Either your struct
becomes generic over `F: Fn(A) -> R` (and now `F` spreads through your entire
API), or you reach for `Box<dyn Fn>` (and lose `Clone`, async, and HRTB).

`closed` fixes that. Every callable becomes a concrete type — no generics
leaking, no `for<'a, 'b>` boilerplate, no manual `Box::pin`.

## Before / after

### Sync + `Clone`

```rust
// Without closed.
struct S {
    cb: Arc<dyn Fn(i32) -> i32 + Send + Sync>,
}

// With closed.
struct S {
    cb: Func<(i32,), i32>,
}
```

### HRTB (borrowed arguments)

```rust
// Without closed — `for<'a, 'b>` is easy to get wrong.
let cb: Arc<dyn for<'a, 'b> Fn(&'a Event, &'b mut World) + Send + Sync>
    = Arc::new(|e, w| { /* ... */ });

// With closed — write the types, get the bounds.
let cb: func!(&Event, &mut World) = (|e, w| { /* ... */ }).into();
```

### Async

```rust
// Without closed.
let cb:
    Arc<dyn Fn(u32) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync>
    = Arc::new(|id| Box::pin(async move { format!("{id}") }));

// With closed.
let cb: func!(u32 -> String; async) = (|id| promise! { format!("{id}") }).into();
```

## The three flavors

Mirroring Rust's `Fn` family:

| `closed` | Standard library | `Clone` | Invocation |
|---|---|---|---|
| `Func<(A, B), O>` | `Fn(A, B) -> O` | Yes | `.invoke((a, b))` |
| `FuncMut<(A, B), O>` | `FnMut(A, B) -> O` | No | `.invoke_mut((a, b))` |
| `FuncOnce<(A, B), O>` | `FnOnce(A, B) -> O` | No | `.invoke_once((a, b))` |

Each has a `*Ref<'a>` twin (`FuncRef<'a, ...>`) for capturing borrowed data.

## Features

- **`Clone` via `Arc`.** Cloning a `Func` is one atomic increment.
- **HRTB by default.** `func!(&str -> usize)` expands to
  `Func<Lend<(Ref<str>,)>, usize>` — i.e.
  `Arc<dyn for<'a> Fn(&'a str) -> usize + Send + Sync>`.
- **Async built in.** `func!(u32 -> String; async)` boxes the returned
  future for you. `promise!` wraps any block into a `PromisePin`.
- **Mixed owned + borrowed.** `func!(String, &str -> String)` works.
- **`no_std` compatible.** Disable the `std` feature; `alloc` is required.
- **Safe.** `#![forbid(unsafe_code)]`.
- **Stable Rust.** No nightly features.
- **Arity up to 8.**

## Example

```rust
use closed::{func, promise};

// Sync, owned.
let add: func!(i32, i32 -> i32) = (|a, b| a + b).into();
assert_eq!(add.invoke((3, 4)), 7);

// HRTB, borrowed.
let join: func!(&_, &_ -> String) = (|a: &str, b: &str| format!("{a}{b}")).into();

assert_eq!(join.invoke(("hello", "world")), "helloworld");

// Async.
let double: func!(u32 -> u32; async) = (|x| promise! { x * 2 }).into();
// double.invoke((21,)).await == 42

// Mutable state.
let mut count = 0;
let mut counter: func!(-> i32; mut) = (move || { count += 1; count }).into();

assert_eq!(counter.invoke_mut(()), 1);
assert_eq!(counter.invoke_mut(()), 2);
```

## When *not* to use `closed`

If you have one or two owned closures and don't need `Clone` or async,
`Arc<dyn Fn>` is simpler and dependency-free. `closed` pays off when:

- you need HRTB (`&Event`, `&mut World`) **and** `Clone`,
- you need async callables stored in structs,
- you have three or more callable fields and want a uniform API,
- you're in `no_std + alloc`.

## Documentation

- API: <https://docs.rs/closed>
- Tests: [`crates/closed/tests/`](https://github.com/vi-is-ramen/closed/tree/main/crates/closed/tests)

## MSRV

Rust **1.88** (Edition 2024).

## License

Dual-licensed under [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.
