//! Core types: [`Callable`], [`CallableMut`], [`CallableOnce`], and aliases.

use crate::traits::{
    CallableArgs, IntoAsyncCallable, IntoAsyncCallableMut,
    IntoAsyncCallableOnce, IntoCallable, IntoCallableMut, IntoCallableOnce,
    ProjectOutput,
};
use alloc::{boxed::Box, sync::Arc};
use core::{future::Future as CoreFuture, pin::Pin};

/// A boxed, [`Send`], `'static` future producing a value of type `T`.
pub type Future<T> = Pin<Box<dyn CoreFuture<Output = T> + Send + 'static>>;

/// The never type (`!`), projected through a stable trait.
pub type Never = <fn() -> ! as ProjectOutput>::Ty;

// --- Callable ------------------------------------------------------------

/// A type-erased, owning, cloneable, thread-safe callable object.
///
/// This is the equivalent of C++'s `std::function`. It wraps any
/// [`Fn`] closure (or function pointer) and stores it behind an [`Arc`].
///
/// Because it is backed by an [`Arc`], cloning a `Callable` is cheap
/// (a single atomic increment) and shares the underlying closure.
///
/// `Callable` is [`Send`] and [`Sync`] whenever the wrapped closure is.
pub struct Callable<Args: CallableArgs, Output: 'static> {
    pub(crate) inner: Arc<Args::DynFn<Output>>,
}

impl<Args: CallableArgs, Output: 'static> Clone for Callable<Args, Output> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Args: CallableArgs, Output: 'static> Callable<Args, Output> {
    /// Constructs a `Callable` from any [`Fn`] closure or function pointer.
    ///
    /// # Example
    /// ```rust
    /// use closed::Callable;
    /// let c: Callable<(i32,), i32> = Callable::new(|x| x * 2);
    /// ```
    pub fn new<Func>(f: Func) -> Self
    where
        Func: IntoCallable<Args, Output>,
    {
        f.into_callable()
    }
}

impl<Args: CallableArgs, Output: 'static> Callable<Args, Future<Output>> {
    /// Constructs an async `Callable` from a closure that returns a [`Future`].
    ///
    /// The closure will be called, and its returned future will be
    /// boxed and pinned into a [`Future`].
    pub fn new_async<Func>(f: Func) -> Self
    where
        Func: IntoAsyncCallable<Args, Output>,
    {
        f.into_async_callable()
    }
}

// --- CallableMut ---------------------------------------------------------

/// A type-erased, owning callable object that requires `&mut self`.
///
/// This is the equivalent of a boxed [`FnMut`] closure. It is not
/// cloneable, as the underlying closure may hold mutable state.
///
/// Unlike [`Callable`], it is backed by a [`Box`], so it cannot be
/// shared across threads via `Arc`. However, it is [`Send`] if the
/// underlying closure is.
pub struct CallableMut<Args: CallableArgs, Output: 'static> {
    pub(crate) inner: Box<Args::DynFnMut<Output>>,
}

impl<Args: CallableArgs, Output: 'static> CallableMut<Args, Output> {
    /// Constructs a `CallableMut` from any [`FnMut`] closure.
    pub fn new<Func>(f: Func) -> Self
    where
        Func: IntoCallableMut<Args, Output>,
    {
        f.into_callable_mut()
    }
}

impl<Args: CallableArgs, Output: 'static> CallableMut<Args, Future<Output>> {
    /// Constructs an async `CallableMut` from a closure returning a [`Future`].
    pub fn new_async<Func>(f: Func) -> Self
    where
        Func: IntoAsyncCallableMut<Args, Output>,
    {
        f.into_async_callable_mut()
    }
}

// --- CallableOnce --------------------------------------------------------

/// A type-erased, owning callable object that consumes `self` when invoked.
///
/// This is the equivalent of a boxed [`FnOnce`] closure. It is not
/// cloneable, and it can only be invoked once.
pub struct CallableOnce<Args: CallableArgs, Output: 'static> {
    pub(crate) inner: Box<Args::DynFnOnce<Output>>,
}

impl<Args: CallableArgs, Output: 'static> CallableOnce<Args, Output> {
    /// Constructs a `CallableOnce` from any [`FnOnce`] closure.
    pub fn new<Func>(f: Func) -> Self
    where
        Func: IntoCallableOnce<Args, Output>,
    {
        f.into_callable_once()
    }
}

impl<Args: CallableArgs, Output: 'static> CallableOnce<Args, Future<Output>> {
    /// Constructs an async `CallableOnce` from a closure returning a [`Future`].
    pub fn new_async<Func>(f: Func) -> Self
    where
        Func: IntoAsyncCallableOnce<Args, Output>,
    {
        f.into_async_callable_once()
    }
}
