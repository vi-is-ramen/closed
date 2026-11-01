//! Type-erased callable objects for Rust.
//!
//! This crate provides owning, type-erased wrappers around callables,
//! filling the same niche as C++'s `std::function`. It solves the
//! "viral generics" problem: instead of making your structs generic
//! over `F: Fn(...)`, you can store a concrete type like
//! `Callable<(A, B), Output>`.
//!
//! The crate provides three main types, mirroring the standard library's
//! `Fn` trait family:
//!
//! - [`Callable`] maps to [`Fn`]. It is cloneable (backed by [`Arc`]) and
//!   can be shared across threads.
//! - [`CallableMut`] maps to [`FnMut`]. It requires `&mut self` to invoke
//!   and is backed by [`Box`].
//! - [`CallableOnce`] maps to [`FnOnce`]. It consumes `self` upon
//!   invocation and is backed by [`Box`].
//!
//! # Argument tuples
//!
//! Following the convention of [`core::ops::Fn`], the arguments are always
//! represented as a tuple. A unary function takes `(A,)`, a binary takes
//! `(A, B)`, and a nullary function takes `()`.
//!
//! # Example
//!
//! ```rust
//! use closed::Callable;
//!
//! // Create a Callable from a closure.
//! let add: Callable<(i32, i32), i32> = Callable::new(|a, b| a + b);
//!
//! // Invoke it.
//! assert_eq!(add.invoke((3, 4)), 7);
//!
//! // Clone it (shares the same underlying closure via Arc).
//! let add2 = add.clone();
//! assert_eq!(add2.invoke((10, 20)), 30);
//! ```
//!
//! # Asynchronous callables
//!
//! There is no separate trait for async closures. Instead, an async
//! callable is just a [`Callable`] whose output is a boxed [`Future`].
//!
//! ```rust
//! use closed::{Callable, Future};
//!
//! let f: Callable<(), Future<i32>> = Callable::new_async(async || 42);
//! // let v = f.invoke(()).await; // requires an async runtime
//! ```
//!
//! # `no_std` support
//!
//! This crate is `#![no_std]` compatible. Disable the default `std`
//! feature in your `Cargo.toml`. The `alloc` crate is required.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod stdfn;
pub mod traits;
pub mod types;

pub use traits::{
    CallableArgs, IntoAsyncCallable, IntoAsyncCallableMut,
    IntoAsyncCallableOnce, IntoCallable, IntoCallableMut, IntoCallableOnce,
    ProjectOutput,
};
pub use types::{Callable, CallableMut, CallableOnce, Future, Never};
