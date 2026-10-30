//! Callable-object abstraction.

/// A type that can be invoked with arguments of type `Args`.
///
/// This trait is the type-erased representation of "something callable",
/// mirroring the role of a function-object in C++. Implementations are
/// provided automatically for all `Fn` closures and function pointers of
/// arity up to 12 (the maximum supported by the standard library), and may
/// be implemented manually for user-defined types:
///
/// ```rust
/// use closed::Callable;
///
/// struct Adder(i32);
///
/// impl Callable<(i32,)> for Adder {
///     type Output = i32;
///     fn invoke(&self, (x,): (i32,)) -> i32 {
///         self.0 + x
///     }
/// }
///
/// let add_2 = Adder(2);
/// assert_eq!(add_2.invoke((40,)), 42);
/// ```
///
/// # Argument tuple convention
///
/// `Args` is always a tuple, matching the convention used by
/// [`std::ops::Fn`]. See the [crate-level documentation](crate) for details.
///
/// # Asynchrony
///
/// There is no separate trait for `async` callables. An asynchronous
/// callable is simply a [`Callable`] whose [`Output`](Callable::Output)
/// implements [`core::future::Future`].
pub trait Callable<Args> {
    /// The type produced by an invocation.
    type Output;

    /// Invoke the callable with the given argument tuple.
    fn invoke(&self, args: Args) -> Self::Output;
}
