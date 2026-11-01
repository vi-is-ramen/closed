# `closed`

[![Crates.io](https://img.shields.io/crates/v/closed.svg)](https://crates.io/crates/closed)
[![Documentation](https://docs.rs/closed/badge.svg)](https://docs.rs/closed)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> Type-erased callable objects for Rust, modeled after C++'s `std::function`.

`closed` provides owning, type-erased wrappers around callables. It fills the
same niche as `std::function` in C++: a value that carries a function of a
specific signature, independent of its concrete type.

The crate solves the "viral generics" problem. Instead of making your structs
generic over `F: Fn(A) -> R` (which infects your entire codebase), or boxing
them as `Box<dyn Fn(A) -> R>` (which loses `Clone` and makes async a nightmare),
you can store a single concrete type.

## Why

Rust's `Fn` traits are excellent for generic code, but they cannot be easily
stored in struct fields:
*   `Box<dyn Fn(A) -> R>` is not `Clone`.
*   `Box<dyn Fn(A) -> R>` cannot easily represent asynchronous callables.
*   Making your struct generic over `F` spreads like a virus through your API.
*   Using a custom `Callable` trait for type erasure causes LLVM to spill
    arguments to the stack during indirect calls.

`closed` solves this by using Rust's built-in `Fn` traits under the hood via a
specialized tuple mapping. Because the final indirect call uses
`extern "rust-call"`, LLVM keeps the arguments in CPU registers (no stack
spilling).

## The Three Flavors

Mirroring Rust's standard library, `closed` provides three distinct types:

| `closed` | Standard Library | Clone | Invocation |
| :--- | :--- | :--- | :--- |
| `Callable` | `Fn` | Yes | `.invoke(args)` |
| `CallableMut` | `FnMut` | No | `.invoke_mut(args)` |
| `CallableOnce`| `FnOnce` | No | `.invoke_once(args)`|

- **`Callable`** is thread-safe (`Send + Sync`) and cheaply cloneable.
- **`CallableMut`** allows capturing mutable state.
- **`CallableOnce`** consumes itself upon invocation.

## Examples

### Basic `Callable` (Fn)

```rust
use closed::Callable;

let add: Callable<(i32, i32), i32> = Callable::new(|a, b| a + b);

// Invoke it.
assert_eq!(add.invoke((3, 4)), 7);

// Clone it (cheaply, via Arc).
let add2 = add.clone();
assert_eq!(add2.invoke((10, 20)), 30);
```

### `CallableMut` (FnMut)

```rust
use closed::CallableMut;

let mut count = 0;
let mut counter: CallableMut<(), i32> = CallableMut::new(move || {
    count += 1;
    count
});

assert_eq!(counter.invoke_mut(()), 1);
assert_eq!(counter.invoke_mut(()), 2);
```

### `CallableOnce` (FnOnce)

```rust
use closed::CallableOnce;

let message = String::from("hello");
let consume: CallableOnce<(), String> = CallableOnce::new(move || message);

// Consumes the closure and the captured String.
assert_eq!(consume.invoke_once(()), "hello");
```

### Asynchronous Callables

There is no separate trait or type for `async`. An asynchronous callable is
simply a callable whose output is a boxed `Future`. 

```rust
use closed::{Callable, Future};

let fetch_data: Callable<(u32,), Future<String>> = 
    Callable::new_async(|id| async move {
        // ... async work ...
        format!("Data for {}", id)
    });

// Requires an async runtime (e.g., tokio)
// let result = fetch_data.invoke((42,)).await;
```

## Argument tuple convention

As in `std::ops::Fn`, the `Args` parameter is always a **tuple** of the
callable's arguments:

| Callable shape | `Args` type |
|---|---|
| `Fn() -> O` | `()` |
| `Fn(A) -> O` | `(A,)` |
| `Fn(A, B) -> O` | `(A, B)` |

This gives a single `.invoke(args)` signature for every arity. It also means
that a unary callable is invoked as `f.invoke((x,))`, not `f.invoke(x)`. This is
the same punctuation cost that `Fn` imposes on generic code.

## Features

*   **Zero-cost type erasure.** No stack spilling of arguments.
*   **`Clone` via `Arc`.** Cloning a `Callable` is a single atomic increment.
*   **Async support.** `new_async` boxes any `impl Future` into
    `Pin<Box<dyn Future + Send>>`.
*   **Arity up to 12.** Matches the standard library's maximum tuple size.
*   **`no_std` compatible.** Disable the `std` feature; `alloc` is required.
*   **Stable Rust.** Works perfectly on stable (with optional nightly features
    for the never type).

## `no_std`

`closed` is `#![no_std]`-compatible when the default `std` feature is disabled.
`alloc` is required.

```toml
closed = { version = "0.0.2", default-features = false }
```

## MSRV

Rust **1.85** (Edition 2024).

## License

Dual-licensed under [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
