# closed

[![Crates.io](https://img.shields.io/crates/v/closed.svg)](https://crates.io/crates/closed)
[![Documentation](https://docs.rs/closed/badge.svg)](https://docs.rs/closed)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Type-erased callable objects for Rust, modeled after C++'s
`std::function`.

`closed` provides an owning, cloneable, type-erased callable,
`Function<Args, Output>`, that can wrap any closure, function
pointer, or user-defined callable. It occupies the same niche as
`std::function<R(Args...)>` in C++: a value carrying a callable of
a fixed signature, independent of the concrete type behind it.

```rust
use closed::Function;

let f: Function<(i32, i32), i32> = Function::new(|a, b| a + b);
assert_eq!(f.invoke((3, 4)), 7);
```

## How it works

Two building blocks:

*   **`Callable<Args>`**: a trait with one method,
    `invoke(&self, args) -> Output`. Implemented automatically for
    every `Fn` closure and function pointer of arity up to 12, and
    implementable manually for your own types.
*   **`Function<Args, Output>`**: the type-erased wrapper.
    Internally an
    `Arc<dyn Callable<Args, Output = Output> + Send + Sync>`, so
    cloning is a single atomic increment and the callable is
    shared, not duplicated.

`Args` is always a tuple, matching `std::ops::Fn`: `()`, `(A,)`,
`(A, B)`, and so on. This yields a single `invoke(args)` method
for every arity. The `Future<O>` alias names a boxed, `Send`,
`'static` future, the canonical output type for async callables.
The `Never` alias names the never type and is available on stable
through an associated-type projection.

## Examples

### Free functions, closures, and custom types

```rust
use closed::{Callable, Function};

fn double(x: i32) -> i32 { x * 2 }
let f = Function::new(double);
assert_eq!(f.invoke((21,)), 42);

let n = 10;
let g = Function::new(move |x: i32| x + n);
assert_eq!(g.invoke((5,)), 15);

struct Triple;
impl Callable<(i32,)> for Triple {
    type Output = i32;
    fn invoke(&self, (x,): (i32,)) -> i32 { x * 3 }
}
let h = Function::new(Triple);
assert_eq!(h.invoke((2,)), 6);
```

### Wrapping a `Function` preserves its type

As in C++, `Function::new(existing_function)` produces a value
of the **same** type. No `Function<Function<...>>` nesting occurs.

```rust
let f: Function<(i32,), i32> = Function::new(|x| x * 2);
let g: Function<(i32,), i32> = Function::new(f);
assert_eq!(g.invoke((21,)), 42);
```

### Async callables

There is no separate trait for async. An async callable is a
`Callable` whose output is a `Future`. `Function::new_async`
handles the boxing for you.

```rust
use closed::{Function, Future};

let f = Function::<(), Future<i32>>::new_async(async || 42);
let v: i32 = f.invoke(()).await;
assert_eq!(v, 42);
```

### Macros for signatures

The `Function!` and `Callable!` macros share the names of the
struct and the trait respectively, and expand to the corresponding
type or bound.

```rust
use closed::{Callable, Function};

fn sum<F: Callable!(i32, i32 -> i32)>(f: F) -> i32 {
    f.invoke((40, 2))
}

let add: Function!(i32, i32 -> i32) = Function::new(|a, b| a + b);
assert_eq!(sum(add), 42);
```

`Function!(A, B -> O)` expands to `Function<(A, B), O>`.
`Callable!(A, B -> O)` expands to
`Callable<(A, B), Output = O>`, valid in trait-bound,
`impl Trait`, and `dyn` positions.

### Nightly call syntax

On a nightly toolchain, `Function` implements `Fn`, `FnMut`, and
`FnOnce`, so the standard call syntax works.

```rust
let f = Function::new(|a: i32, b: i32| a + b);
assert_eq!(f(3, 4), 7);
```

The toolchain is detected at build time via `rustversion`. No
feature flag is needed and no configuration is required.

## `no_std`

`closed` is `#![no_std]`-compatible when the default `std`
feature is disabled. `alloc` is required.

```toml
closed = { version = "0.0.1", default-features = false }
```

## What it doesn't do

*   **No `FnMut`/`FnOnce` support on stable.** Only `Fn`
    closures are covered by the automatic impls, because
    `Function` invokes through `&self`. `FnMut` and `FnOnce` are
    provided on nightly via the `fn_traits` feature.
*   **No `target_type()` or `target<T>()`.** Unlike
    `std::function`'s RTTI, `closed::Function` does not expose the
    wrapped type. Add it yourself if you need it.
*   **No automatic `Fn` implementation on stable.** Blanket
    `impl<T: Callable> Fn for T` is not possible due to coherence
    rules against the standard library's own `Fn` impls. The
    nightly `Fn` impl therefore targets `Function` specifically.

## Status

Pre-1.0. The public API is intended to be stable in shape;
details may shift in minor versions.

## MSRV

Rust **1.85** (edition 2024).

## License

Dual-licensed under
[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
