//! General types, used by `closed` itself and exported to the public API.

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use crate::private::sig::Signature;

/// A boxed, `Send`, `'a` future producing a value of type `T`.
pub type PromisePin<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A boxed, `Send`, `'static` future producing a value of type `T`.
pub type Promise<T> = PromisePin<'static, T>;

/// Inhabited type that can not be constructed. Used as "this function never
/// returns" marker if used in return type position.
pub type Never = <fn() -> ! as Signature<()>>::Out;
