//! Core types: [`Function`] and its supporting aliases.

use crate::traits::Callable;
use ::alloc::{boxed::Box, sync::Arc};
use ::core::{future::Future as CoreFuture, marker::Send, pin::Pin};

/// A boxed, [`Send`] future with an optional lifetime parameter.
///
/// This is the un-erased form. Most call sites want [`Future`], which fixes
/// the lifetime to `'static` — the only form that can be stored inside a
/// type-erased [`Function`].
pub type BaseFuture<'a, T> = Pin<Box<dyn CoreFuture<Output = T> + Send + 'a>>;

/// A boxed, [`Send`], `'static` future producing a value of type `T`.
///
/// This alias is the canonical [`Output`](Callable::Output) parameter for an
/// asynchronous [`Function`], and is the type produced by
/// [`Function::new_async`].
pub type Future<T> = BaseFuture<'static, T>;

/// A type-erased, owning, cloneable callable object.
///
/// `Function<Args, Output>` is the Rust analogue of C++'s
/// `std::function<Output(Args...)>`: it can wrap any closure, function
/// pointer, or user-defined [`Callable`] with matching argument and output
/// types, and it can be passed around, cloned, and shared across threads.
///
/// # Storage
///
/// Internally, a `Function` holds an
/// `Arc<dyn Callable<Args, Output = Output> + Send + Sync>`. Two
/// consequences follow:
///
/// - `Function` is [`Clone`], and cloning is cheap (a single atomic
///   increment on the shared [`Arc`]).
/// - `Function` is [`Send`] and [`Sync`] whenever the wrapped callable is.
///
/// # Example
///
/// ```rust
/// use closed::Function;
///
/// let f = Function::new(|x: i32| x + 1);
/// let g = f.clone();
///
/// assert_eq!(f.invoke((1,)), 2);
/// assert_eq!(g.invoke((1,)), 2);
/// ```
///
/// # Idempotence of wrapping
///
/// Wrapping an existing `Function` in another `Function` produces a value
/// of the same type, matching the behavior of `std::function{existing}` in
/// C++:
///
/// ```rust
/// use closed::Function;
///
/// let f: Function<(i32,), i32> = Function::new(|x| x * 2);
/// let g: Function<(i32,), i32> = Function::new(f);
/// assert_eq!(g.invoke((21,)), 42);
/// ```
pub struct Function<Args, Output> {
    inner: Arc<dyn Callable<Args, Output = Output> + Send + Sync>,
}

/// Cloning a `Function` clones the internal [`Arc`], sharing the wrapped
/// callable rather than duplicating it.
///
/// This impl is written by hand rather than derived: `#[derive(Clone)]`
/// would insert unnecessary `Args: Clone` and `Output: Clone` bounds, which
/// do not reflect the actual requirements.
impl<Args, Output> Clone for Function<Args, Output> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Args, Output> Function<Args, Output> {
    /// Construct a `Function` from any value implementing [`Callable`] with
    /// the given `Args` and `Output`.
    ///
    /// The wrapped value is stored behind an [`Arc`], so `f` is moved into
    /// the `Function`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use closed::Function;
    ///
    /// let f = Function::new(|x: i32| x * 2);
    /// assert_eq!(f.invoke((21,)), 42);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Callable<Args, Output = Output> + Send + Sync + 'static,
    {
        Self { inner: Arc::new(f) }
    }

    /// Invoke the underlying callable with the given argument tuple.
    ///
    /// Equivalent to calling [`Callable::invoke`] on the `Function` itself.
    pub fn invoke(&self, args: Args) -> Output {
        self.inner.invoke(args)
    }
}

/// On stable toolchains, `Function` implements [`Callable`] directly.
///
/// On nightly toolchains, this impl is replaced by the blanket impl in
/// [`crate::stdfn`], which applies to `Function` via the
/// [`Fn`](core::ops::Fn) implementation provided in
/// [`crate::nightly_impls`]. Keeping only one of the two avoids a coherence
/// conflict (`E0119`) on nightly.
#[cfg(not(nightly))]
impl<Args, Output> Callable<Args> for Function<Args, Output> {
    type Output = Output;
    fn invoke(&self, args: Args) -> Output {
        self.inner.invoke(args)
    }
}

impl<Args, O> Function<Args, Future<O>>
where
    O: 'static,
{
    /// Construct a `Function` from an asynchronous callable.
    ///
    /// The provided callable `f` may be any `Fn(Args) -> Fut`; the `Fut`
    /// it produces is boxed into a [`Future<O>`] on each invocation. This
    /// is how closures returning `impl Future` — including `async`
    /// closures — are adapted to the erased, `Function`-compatible form.
    ///
    /// Unlike [`Function::new`], which takes a [`Callable`] directly,
    /// `new_async` accepts any `Fn` — including `async` closures — because
    /// the boxing required to unify `impl Future` with
    /// `dyn Future + Send` is performed inside the constructor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use closed::{Function, Future};
    ///
    /// let f = Function::<(), Future<i32>>::new_async(async || 42);
    /// ```
    pub fn new_async<F, Fut>(f: F) -> Self
    where
        F: Callable<Args, Output = Fut> + Send + Sync + 'static,
        Fut: CoreFuture<Output = O> + Send + 'static,
    {
        Self {
            inner: Arc::new(AsyncWrapper(f)),
        }
    }
}

// --- internal -----------------------------------------

/// Adapter that turns a synchronous `Callable` producing a future into a
/// `Callable` whose output is the erased, boxed form.
struct AsyncWrapper<F>(F);

impl<F, Args, Fut, O> Callable<Args> for AsyncWrapper<F>
where
    F: Callable<Args, Output = Fut>,
    Fut: CoreFuture<Output = O> + Send + 'static,
{
    type Output = Future<O>;
    fn invoke(&self, args: Args) -> Future<O> {
        Box::pin(self.0.invoke(args))
    }
}

// --- funny --------------------------------------------

/// The never type, made available on stable channels.
///
/// The projection `<fn() -> ! as Callable<()>>::Output` normalizes to `!`,
/// which is legal to name through an associated-type projection even though
/// the `never_type` feature gate remains unstable for syntactic uses of `!`
/// in type position.
///
/// This is useful for expressing hook signatures such as
/// `Function<(), Never>` for callbacks that must not return.
pub type Never = <fn() -> ! as Callable<()>>::Output;
