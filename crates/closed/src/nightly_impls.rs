//! Nightly-only implementations of [`Fn`], [`FnMut`], and [`FnOnce`] for
//! [`Function`].
//!
//! These impls allow a [`Function`] to be called using the standard call
//! syntax, matching the ergonomics of C++'s `std::function`:
//!
//! ```rust,ignore
//! // Requires nightly.
//! let f = Function::new(|a: i32, b: i32| a + b);
//! assert_eq!(f(3, 4), 7);
//! ```
//!
//! The module is enabled by the `nightly` `cfg` flag, which is set by
//! `build.rs` when the crate is compiled with a nightly toolchain.

use crate::Function;
use core::{
    marker::Tuple,
    ops::{Fn, FnMut, FnOnce},
};

impl<Args: Tuple, Output> FnOnce<Args> for Function<Args, Output> {
    type Output = Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.invoke(args)
    }
}

impl<Args: Tuple, Output> FnMut<Args> for Function<Args, Output> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.invoke(args)
    }
}

impl<Args: Tuple, Output> Fn<Args> for Function<Args, Output> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.invoke(args)
    }
}
