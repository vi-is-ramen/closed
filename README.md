# `closed`

> Type-erased callable objects for Rust, modeled after C++'s
> `std::function`.

`closed` provides an owning, cloneable, type-erased callable,
`Function<Args, Output>`, that can wrap any closure, function pointer, or
user-defined callable. It fills the same niche as
`std::function<R(Args...)>` in C++: a value that carries a callable of a
specific signature, independent of its concrete type.

---

## Why

Rust's `Fn` traits are excellent for generic code, but they cannot be
stored directly in a struct field without either:

*   making the struct generic over `F` (viral generics), or
*   boxing as `Box<dyn Fn(A) -> R>`, which loses `Clone`, cannot easily
    represent async callables, and treats `Fn`, `FnMut`, and `FnOnce`
    as separate types.

`closed` consolidates all of this behind a single type with a single
invocation method:

```rust
use closed::Function;

let f: Function<(i32, i32), i32> = Function::new(|a, b| a + b);
assert_eq!(f.invoke((3, 4)), 7);
```

`Function` is `Clone` (backed by `Arc`), `Send`, and `Sync` whenever the
wrapped callable is. It handles synchronous and asynchronous callables
uniformly: an async callable is simply a `Callable` whose output is a
`Future`.

## Workspace layout

| Crate | Purpose |
|---|---|
| [`closed`](crates/closed) | Public API: types, traits, macros. |
| [`closed-proc-macros`](crates/closed-proc-macros) | For internal use only. |

## Features

*   **Single invocation method.** `Callable::invoke(&self, args: Args)`
    with `Args` as a tuple, matching the convention of `std::ops::Fn`.
*   **Blanket impls for `Fn`.** Any closure, function pointer, or
    `fn`-item of arity up to 12 implements `Callable` automatically.
    No wrapper needed.
*   **Manual impls.** User-defined callable types implement `Callable`
    directly and become usable with `Function`.
*   **`Clone` via `Arc`.** Cloning a `Function` is a single atomic
    increment; the wrapped callable is shared, not duplicated.
*   **Async support without a second trait.** `Function::new_async`
    boxes a `Fn(Args) -> impl Future` into a
    `Function<Args, Future<O>>`. All async callables go through the
    same path as synchronous ones.
*   **Ergonomic type macros.** `Function!(A, B -> O)` expands to
    `Function<(A, B), O>`; `Callable!(A, B -> O)` expands to a bound
    `Callable<(A, B), Output = O>`, valid in bound, `impl Trait`,
    and `dyn` positions.
*   **Nightly call syntax.** With a nightly toolchain, `Function`
    additionally implements `Fn`, `FnMut`, and `FnOnce`, enabling
    `f(1, 2)` instead of `f.invoke((1, 2))`. Detected automatically
    at build time via `rustversion`.
*   **`no_std` compatible.** Disable the default `std` feature;
    `alloc` is required.

## Example

```rust
use closed::{Callable, Function};

// A free function.
fn double(x: i32) -> i32 { x * 2 }
let f = Function::new(double);
assert_eq!(f.invoke((21,)), 42);

// A closure with captures.
let n = 10;
let g = Function::new(move |x: i32| x + n);
assert_eq!(g.invoke((5,)), 15);

// A user-defined callable.
struct Triple;
impl Callable<(i32,)> for Triple {
    type Output = i32;
    fn invoke(&self, (x,): (i32,)) -> i32 { x * 3 }
}
let h = Function::new(Triple);
assert_eq!(h.invoke((2,)), 6);

// Wrapping a Function in a Function preserves the type, as in C++.
let i: Function<(i32,), i32> = Function::new(f);
assert_eq!(i.invoke((21,)), 42);
```

Asynchronous callables use `Function::new_async`:

```rust
use closed::{Function, Future};

let f = Function::<(), Future<i32>>::new_async(async || 42);
let v: i32 = f.invoke(()).await;
assert_eq!(v, 42);
```

Macros shorten type annotations:

```rust
// `Callable!` and `Function!` are proc macros sharing the names of
// the trait and struct, so a single `use` imports both.
use closed::{Callable, Function};

fn sum<F: Callable!(i32, i32 -> i32)>(f: F) -> i32 {
    f.invoke((40, 2))
}

let add: Function!(i32, i32 -> i32) = Function::new(|a, b| a + b);
assert_eq!(sum(add), 42);
```

## Argument tuple convention

As in `std::ops::Fn`, `Args` is always a **tuple** of the callable's
arguments:

| Callable shape | `Args` |
|---|---|
| `Fn() -> O` | `()` |
| `Fn(A) -> O` | `(A,)` |
| `Fn(A, B) -> O` | `(A, B)` |
| `Fn(A, B, C) -> O` | `(A, B, C)` |

This gives a single `invoke(args)` signature for every arity. It also
means that a unary callable is invoked as `f.invoke((x,))`, not
`f.invoke(x)`. This is the same punctuation cost that `Fn` imposes on
generic code.

## Stability and status

Pre-1.0. The public API is subject to change between minor versions.
`Function`, `Callable`, `Future`, `Never`, and the two type macros
are considered the intended public surface; anything else is
implementation detail.

## Minimum supported Rust version

*   Rust **1.85** (edition 2024) on stable.
*   Nightly features (`Fn` impls) are opt-in by virtue of using a
    nightly toolchain; the crate builds and passes all non-nightly
    tests on stable.

## License

Dual-licensed under [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual-licensed as above, without any
additional terms or conditions.
