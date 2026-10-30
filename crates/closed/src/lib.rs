//! Type-erased callable objects, modeled after C++'s `std::function`.
//!
//! [`Function<Args, Output>`] is an owning, cloneable, type-erased callable
//! that can wrap any [`Fn`] closure, function pointer, or user-defined type
//! implementing [`Callable`]. It occupies the same niche as
//! `std::function<R(Args...)>` in C++.
//!
//! # Overview
//!
//! The crate is built around three types:
//!
//! - [`Callable`]: a trait describing a callable object. Implementations are
//!   provided automatically for all `Fn` closures and function pointers via
//!   blanket impls generated at compile time, and may be implemented manually
//!   for user-defined types.
//! - [`Function`]: the type-erased, owning wrapper. Internally an
//!   `Arc<dyn Callable<...> + Send + Sync>`, giving `Clone` and
//!   `Send + Sync` for free when the wrapped callable is `Send + Sync`.
//! - [`Future`]: a convenience alias for boxed, `Send` futures, intended
//!   as the [`Output`](Callable::Output) type for `async` callables.
//!
//! # Example
//!
//! ```rust
//! use closed::{Callable, Function};
//!
//! // Function pointers.
//! fn double(x: i32) -> i32 { x * 2 }
//! let f = Function::new(double);
//! assert_eq!(f.invoke((21,)), 42);
//!
//! // Closures with captures.
//! let n = 10;
//! let g = Function::new(move |x: i32| x + n);
//! assert_eq!(g.invoke((5,)), 15);
//!
//! // User-defined callables.
//! struct Triple;
//! impl Callable<(i32,)> for Triple {
//!     type Output = i32;
//!     fn invoke(&self, (x,): (i32,)) -> i32 { x * 3 }
//! }
//! let h = Function::new(Triple);
//! assert_eq!(h.invoke((2,)), 6);
//! ```
//!
//! # Argument tuple convention
//!
//! As in [`std::ops::Fn`], the `Args` parameter is a **tuple** of the
//! callable's arguments. A nullary callable has `Args = ()`, a unary
//! callable has `Args = (A,)`, a binary callable has `Args = (A, B)`, and
//! so on.
//!
//! # Nightly features
//!
//! When the crate is compiled with a nightly toolchain, detected at build
//! time via `rustversion` in `build.rs`, additional implementations of
//! [`Fn`], [`FnMut`], and [`FnOnce`] are provided for [`Function`]. This
//! enables calling `Function` instances with the standard call syntax
//! (`f(1, 2)`) instead of the explicit `.invoke((1, 2))` form.
//!
//! ```rust,ignore
//! // Requires nightly.
//! let f = Function::new(|a: i32, b: i32| a + b);
//! assert_eq!(f(3, 4), 7);
//! ```
//!
//! # `no_std`
//!
//! The crate is `#![no_std]`-compatible when the `std` feature is disabled.
//! It requires the `alloc` crate for `Arc` and `Box`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(fn_traits, unboxed_closures, tuple_trait))]

extern crate alloc;
extern crate self as closed;

#[cfg(nightly)]
mod nightly_impls;
mod stdfn;
pub mod traits;
pub mod types;

pub use closed_proc_macros::{callable as Callable, function as Function};
pub use traits::Callable;
pub use types::{Function, Future, Never};

#[cfg(test)]
mod tests {
    use super::*;

    fn free_fn(x: i32) -> i32 {
        x * 2
    }
    struct Doubler;
    impl Callable<(i32,)> for Doubler {
        type Output = i32;
        fn invoke(&self, (x,): (i32,)) -> i32 {
            x * 2
        }
    }

    #[test]
    fn free_function() {
        let f = Function::new(free_fn);
        assert_eq!(f.invoke((21,)), 42);
    }

    #[test]
    fn capturing_closure() {
        let n = 10;
        let f = Function::new(move |x: i32| x + n);
        assert_eq!(f.invoke((5,)), 15);
    }

    #[test]
    fn custom_callable() {
        let f = Function::new(Doubler);
        assert_eq!(f.invoke((4,)), 8);
    }

    #[test]
    fn function_of_function_is_idempotent() {
        let f: Function<(i32,), i32> = Function::new(free_fn);
        let g: Function<(i32,), i32> = Function::new(f.clone());
        assert_eq!(f.invoke((3,)), g.invoke((3,)));
    }

    #[test]
    fn clone_shares_state() {
        let f = Function::new(|x: i32| x + 1);
        let g = f.clone();
        assert_eq!(f.invoke((1,)), g.invoke((1,)));
    }

    #[tokio::test]
    async fn async_closure() {
        let f = Function::<(), Future<i32>>::new_async(async || 42);
        assert_eq!(f.invoke(()).await, 42);
    }

    #[cfg(nightly)]
    #[test]
    fn nightly_call_syntax() {
        let f = Function::new(|x: i32| x + 1);
        assert_eq!(f(1), 2);

        let g = Function::new(|a: i32, b: i32| a + b);
        assert_eq!(g(3, 4), 7);

        let h = Function::new(|| 42);
        assert_eq!(h(), 42);
    }

    #[test]
    fn function_macro() {
        let f: Function!(i32 -> i32) = Function::new(|x: i32| x + 1);
        assert_eq!(f.invoke((41,)), 42);
    }

    #[test]
    fn function_macro_nullary() {
        let f: Function!(-> i32) = Function::new(|| 42);
        assert_eq!(f.invoke(()), 42);
    }

    #[test]
    fn function_macro_binary() {
        let f: Function!(i32, i32 -> i32) =
            Function::new(|a: i32, b: i32| a + b);
        assert_eq!(f.invoke((3, 4)), 7);
    }
}
