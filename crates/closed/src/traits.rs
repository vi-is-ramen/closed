//! Traits for mapping argument tuples to `dyn Fn*` types, and for
//! constructing the type-erased wrappers.

use crate::types::{Callable, CallableMut, CallableOnce, Future};

/// A trait that maps a tuple of arguments (e.g., `(A, B)`) to the
/// corresponding `dyn Fn`, `dyn FnMut`, and `dyn FnOnce` trait objects.
///
/// This trait is the "glue" that allows [`Callable`] to remain generic
/// over `Args` while still using the compiler's built-in `Fn` traits
/// under the hood. It is implemented for all tuples up to 12 elements.
///
/// You should not need to implement this trait manually.
pub trait CallableArgs {
    /// The corresponding `dyn Fn` type.
    /// For `(A, B)`, this is `dyn Fn(A, B) -> Output + Send + Sync`.
    type DynFn<Output: 'static>: ?Sized + Send + Sync + 'static;

    /// The corresponding `dyn FnMut` type.
    /// For `(A, B)`, this is `dyn FnMut(A, B) -> Output + Send`.
    type DynFnMut<Output: 'static>: ?Sized + Send + 'static;

    /// The corresponding `dyn FnOnce` type.
    /// For `(A, B)`, this is `dyn FnOnce(A, B) -> Output + Send`.
    type DynFnOnce<Output: 'static>: ?Sized + Send + 'static;
}

// --- Sync Constructors ---

/// Helper trait to construct a [`Callable`] from any [`Fn`] closure.
///
/// This trait is implemented automatically for any closure or function
/// pointer that satisfies the bounds.
pub trait IntoCallable<Args: CallableArgs, Output: 'static> {
    /// Converts `self` into a type-erased [`Callable`].
    fn into_callable(self) -> Callable<Args, Output>;
}

/// Helper trait to construct a [`CallableMut`] from any [`FnMut`] closure.
pub trait IntoCallableMut<Args: CallableArgs, Output: 'static> {
    /// Converts `self` into a type-erased [`CallableMut`].
    fn into_callable_mut(self) -> CallableMut<Args, Output>;
}

/// Helper trait to construct a [`CallableOnce`] from any [`FnOnce`] closure.
pub trait IntoCallableOnce<Args: CallableArgs, Output: 'static> {
    /// Converts `self` into a type-erased [`CallableOnce`].
    fn into_callable_once(self) -> CallableOnce<Args, Output>;
}

// --- Async Constructors ---

/// Helper trait to construct an async [`Callable`] from a closure
/// returning a [`Future`].
pub trait IntoAsyncCallable<Args: CallableArgs, Output: 'static> {
    /// Converts `self` into an async [`Callable`].
    fn into_async_callable(self) -> Callable<Args, Future<Output>>;
}

/// Helper trait to construct an async [`CallableMut`] from a closure
/// returning a [`Future`].
pub trait IntoAsyncCallableMut<Args: CallableArgs, Output: 'static> {
    /// Converts `self` into an async [`CallableMut`].
    fn into_async_callable_mut(self) -> CallableMut<Args, Future<Output>>;
}

/// Helper trait to construct an async [`CallableOnce`] from a closure
/// returning a [`Future`].
pub trait IntoAsyncCallableOnce<Args: CallableArgs, Output: 'static> {
    /// Converts `self` into an async [`CallableOnce`].
    fn into_async_callable_once(self) -> CallableOnce<Args, Future<Output>>;
}

// --- Type Projections ---

/// A helper trait to project the never type `!` on stable Rust.
///
/// Since the `never_type` feature is unstable, we cannot write `!` directly
/// in type position. However, we can project it through the output of a
/// function pointer: `fn() -> !`.
pub trait ProjectOutput {
    /// The projected type.
    type Ty;
}

impl<T> ProjectOutput for fn() -> T {
    type Ty = T;
}
