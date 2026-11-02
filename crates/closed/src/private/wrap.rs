//! Main module. contains [`Func`] and it's twins.

use crate::private::sig::{Arg, FnArgs, Lend, Signature};
use crate::private::types::PromisePin;
use alloc::{boxed::Box, sync::Arc};
use core::future::Future;

// --- Static Aliases ---

/// An owning, type-erased `Fn` callable with a `'static` lifetime.
pub type Func<Args, Out> = FuncRef<'static, Args, Out>;
/// An owning, type-erased `FnMut` callable with a `'static` lifetime.
pub type FuncMut<Args, Out> = FuncMutRef<'static, Args, Out>;
/// An owning, type-erased `FnOnce` callable with a `'static` lifetime.
pub type FuncOnce<Args, Out> = FuncOnceRef<'static, Args, Out>;

// --- Structs with Lifetimes ---

/// An owning, type-erased `Fn` callable.
pub struct FuncRef<'a, Args: FnArgs + 'a, Out: 'a> {
    pub(crate) inner: Arc<Args::DynFn<'a, Out>>,
}

impl<'a, Args: FnArgs + 'a, Out: 'a> Clone for FuncRef<'a, Args, Out> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// An owning, type-erased `FnMut` callable.
pub struct FuncMutRef<'a, Args: FnArgs + 'a, Out: 'a> {
    pub(crate) inner: Box<Args::DynFnMut<'a, Out>>,
}

/// An owning, type-erased `FnOnce` callable.
pub struct FuncOnceRef<'a, Args: FnArgs + 'a, Out: 'a> {
    pub(crate) inner: Box<Args::DynFnOnce<'a, Out>>,
}

impl<'a, Args: FnArgs + 'a, Out: 'a> core::fmt::Debug
    for FuncRef<'a, Args, Out>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("fn ref (..) -> ..")
    }
}

impl<'a, Args: FnArgs + 'a, Out: 'a> core::fmt::Debug
    for FuncMutRef<'a, Args, Out>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("fn mut (..) -> ..")
    }
}

impl<'a, Args: FnArgs + 'a, Out: 'a> core::fmt::Debug
    for FuncOnceRef<'a, Args, Out>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("fn (..) -> ..")
    }
}

// --- Signature Implementations ---

impl<'a, Args: FnArgs + 'a, Out: 'a> Signature<Args>
    for FuncRef<'a, Args, Out>
{
    type Out = Out;
}

impl<'a, Args: FnArgs + 'a, Out: 'a> Signature<Args>
    for FuncMutRef<'a, Args, Out>
{
    type Out = Out;
}

impl<'a, Args: FnArgs + 'a, Out: 'a> Signature<Args>
    for FuncOnceRef<'a, Args, Out>
{
    type Out = Out;
}

// --- Method Generation ---

macro_rules! impl_func_methods {
    ($($arg:ident),*) => {
        #[allow(non_snake_case)]
        impl<'a, $($arg: 'a,)* Out: 'a> FuncRef<'a, ($($arg,)*), Out> {
            /// Construct new [`Self`] from the given closure.
            pub fn new<Func>(f: Func) -> Self
            where
                Func: Fn($($arg),*) -> Out + Send + Sync + 'a,
            {
                Self { inner: Arc::new(f) }
            }

            /// Invoke the closure with given arguments tuple.
            #[inline(always)]
            pub fn invoke(&self, ($($arg,)*): ($($arg,)*)) -> Out {
                (self.inner)($($arg),*)
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg: 'a,)* T: 'a>
            FuncRef<'a, ($($arg,)*), PromisePin<'a, T>>
        {
            /// Construct new [`Self`] from the given async closure.
            pub fn new_async<Func, Fut>(f: Func) -> Self
            where
                Func: Fn($($arg),*) -> Fut + Send + Sync + 'a,
                Fut: Future<Output = T> + Send + 'a,
            {
                Self {
                    inner: Arc::new(
                        move |$($arg: $arg),*| -> PromisePin<'a, T> {
                            Box::pin(f($($arg),*))
                        },
                    ),
                }
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg: 'a,)* Out: 'a> FuncMutRef<'a, ($($arg,)*), Out> {
            /// Construct new [`Self`] from the given closure.
            pub fn new<Func>(f: Func) -> Self
            where
                Func: FnMut($($arg),*) -> Out + Send + 'a,
            {
                Self { inner: Box::new(f) }
            }

            /// Invoke the closure with given arguments tuple.
            #[inline(always)]
            pub fn invoke_mut(&mut self, ($($arg,)*): ($($arg,)*)) -> Out {
                (self.inner)($($arg),*)
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg: 'a,)* T: 'a>
            FuncMutRef<'a, ($($arg,)*), PromisePin<'a, T>>
        {
            /// Construct new [`Self`] from the given async closure.
            pub fn new_async<Func, Fut>(mut f: Func) -> Self
            where
                Func: FnMut($($arg),*) -> Fut + Send + 'a,
                Fut: Future<Output = T> + Send + 'a,
            {
                Self {
                    inner: Box::new(
                        move |$($arg: $arg),*| -> PromisePin<'a, T> {
                            Box::pin(f($($arg),*))
                        },
                    ),
                }
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg: 'a,)* Out: 'a> FuncOnceRef<'a, ($($arg,)*), Out> {
            /// Construct new [`Self`] from the given closure.
            pub fn new<Func>(f: Func) -> Self
            where
                Func: FnOnce($($arg),*) -> Out + Send + 'a,
            {
                Self { inner: Box::new(f) }
            }

            /// Invoke the closure with given arguments tuple.
            #[inline(always)]
            pub fn invoke_once(self, ($($arg,)*): ($($arg,)*)) -> Out {
                (self.inner)($($arg),*)
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg: 'a,)* T: 'a>
            FuncOnceRef<'a, ($($arg,)*), PromisePin<'a, T>>
        {
            /// Construct new [`Self`] from the given async closure.
            pub fn new_async<Func, Fut>(f: Func) -> Self
            where
                Func: FnOnce($($arg),*) -> Fut + Send + 'a,
                Fut: Future<Output = T> + Send + 'a,
            {
                Self {
                    inner: Box::new(
                        move |$($arg: $arg),*| -> PromisePin<'a, T> {
                            Box::pin(f($($arg),*))
                        },
                    ),
                }
            }
        }
    };
}

impl_func_methods!();
impl_func_methods!(A);
impl_func_methods!(A, B);
impl_func_methods!(A, B, C);
impl_func_methods!(A, B, C, D);
impl_func_methods!(A, B, C, D, E);
impl_func_methods!(A, B, C, D, E, F);
impl_func_methods!(A, B, C, D, E, F, G);
impl_func_methods!(A, B, C, D, E, F, G, H);

macro_rules! impl_lend_func_methods {
    ($($arg:ident : $lt:lifetime),+) => {
        #[allow(non_snake_case)]
        impl<'f, $($arg: Arg,)+ Out: 'f>
            FuncRef<'f, Lend<($($arg,)+)>, Out>
        {
            /// Construct new [`Self`] from the given closure.
            pub fn new<Func>(f: Func) -> Self
            where
                Func: for<$($lt),+> Fn(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Out + Send + Sync + 'f,
            {
                Self { inner: Arc::new(f) }
            }

            /// Invoke the closure with given arguments tuple.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub fn invoke<$($lt),+>(
                &self,
                args: ($(<$arg as Arg>::Rebind<$lt>,)+),
            ) -> Out {
                let ($($arg,)+) = args;
                (self.inner)($($arg),+)
            }
        }

        #[allow(non_snake_case)]
        impl<'f, $($arg: Arg,)+ T: 'f>
            FuncRef<'f, Lend<($($arg,)+)>, PromisePin<'f, T>>
        {
            /// Construct new [`Self`] from the given async closure.
            pub fn new_async<Func, Fut>(f: Func) -> Self
            where
                Func: for<$($lt),+> Fn(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Fut + Send + Sync + 'f,
                Fut: Future<Output = T> + Send + 'f,
            {
                Self {
                    inner: Arc::new(
                        // Аргументы без аннотаций — сигнатура выводится
                        // из ожидаемого `for<…> Fn(…) -> PromisePin<'f, T>`.
                        move |$($arg),+| -> PromisePin<'f, T> {
                            Box::pin(f($($arg),+))
                        },
                    ),
                }
            }
        }

        #[allow(non_snake_case)]
        impl<'f, $($arg: Arg,)+ Out: 'f>
            FuncMutRef<'f, Lend<($($arg,)+)>, Out>
        {
            /// Construct new [`Self`] from the given closure.
            pub fn new<Func>(f: Func) -> Self
            where
                Func: for<$($lt),+> FnMut(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Out + Send + 'f,
            {
                Self { inner: Box::new(f) }
            }

            /// Invoke the closure with given arguments tuple.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub fn invoke_mut<$($lt),+>(
                &mut self,
                args: ($(<$arg as Arg>::Rebind<$lt>,)+),
            ) -> Out {
                let ($($arg,)+) = args;
                (self.inner)($($arg),+)
            }
        }

        #[allow(non_snake_case)]
        impl<'f, $($arg: Arg,)+ Out: 'f>
            FuncOnceRef<'f, Lend<($($arg,)+)>, Out>
        {
            /// Construct new [`Self`] from the given closure.
            pub fn new<Func>(f: Func) -> Self
            where
                Func: for<$($lt),+> FnOnce(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Out + Send + 'f,
            {
                Self { inner: Box::new(f) }
            }

            /// Invoke the closure with given arguments tuple.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub fn invoke_once<$($lt),+>(
                self,
                args: ($(<$arg as Arg>::Rebind<$lt>,)+),
            ) -> Out {
                let ($($arg,)+) = args;
                (self.inner)($($arg),+)
            }
        }

        #[allow(non_snake_case)]
        impl<'f, $($arg: Arg,)+ T: 'f>
            FuncMutRef<'f, Lend<($($arg,)+)>, PromisePin<'f, T>>
        {
            /// Construct new [`Self`] from the given async closure.
            pub fn new_async<Func, Fut>(mut f: Func) -> Self
            where
                Func: for<$($lt),+> FnMut(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Fut + Send + 'f,
                Fut: Future<Output = T> + Send + 'f,
            {
                Self {
                    inner: Box::new(
                        move |$($arg),+| -> PromisePin<'f, T> {
                            Box::pin(f($($arg),+))
                        },
                    ),
                }
            }
        }

        #[allow(non_snake_case)]
        impl<'f, $($arg: Arg,)+ T: 'f>
            FuncOnceRef<'f, Lend<($($arg,)+)>, PromisePin<'f, T>>
        {
            /// Construct new [`Self`] from the given async closure.
            pub fn new_async<Func, Fut>(f: Func) -> Self
            where
                Func: for<$($lt),+> FnOnce(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Fut + Send + 'f,
                Fut: Future<Output = T> + Send + 'f,
            {
                Self {
                    inner: Box::new(
                        move |$($arg),+| -> PromisePin<'f, T> {
                            Box::pin(f($($arg),+))
                        },
                    ),
                }
            }
        }
    };
}

impl_lend_func_methods!(A: 'x0);
impl_lend_func_methods!(A: 'x0, B: 'x1);
impl_lend_func_methods!(A: 'x0, B: 'x1, C: 'x2);
impl_lend_func_methods!(A: 'x0, B: 'x1, C: 'x2, D: 'x3);
impl_lend_func_methods!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4);
impl_lend_func_methods!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, F: 'x5);
impl_lend_func_methods!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, F: 'x5, G: 'x6);
impl_lend_func_methods!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, F: 'x5, G: 'x6, H: 'x7);

macro_rules! impl_from_tuple {
    ($($arg:ident),*) => {
        impl<'a, $($arg: 'a,)* Out: 'a, Fun>
            From<Fun> for FuncRef<'a, ($($arg,)*), Out>
        where
            Fun: Fn($($arg),*) -> Out + Send + Sync + 'a,
        {
            fn from(f: Fun) -> Self {
                Self { inner: Arc::new(f) }
            }
        }

        impl<'a, $($arg: 'a,)* Out: 'a, Fun>
            From<Fun> for FuncMutRef<'a, ($($arg,)*), Out>
        where
            Fun: FnMut($($arg),*) -> Out + Send + 'a,
        {
            fn from(f: Fun) -> Self {
                Self { inner: Box::new(f) }
            }
        }

        impl<'a, $($arg: 'a,)* Out: 'a, Fun>
            From<Fun> for FuncOnceRef<'a, ($($arg,)*), Out>
        where
            Fun: FnOnce($($arg),*) -> Out + Send + 'a,
        {
            fn from(f: Fun) -> Self {
                Self { inner: Box::new(f) }
            }
        }
    };
}

macro_rules! impl_from_lend {
    ($($arg:ident : $lt:lifetime),+) => {
        impl<'f, $($arg: Arg,)+ Out: 'f, Fun>
            From<Fun> for FuncRef<'f, Lend<($($arg,)+)>, Out>
        where
            Fun: for<$($lt),+> Fn(
                $(<$arg as Arg>::Rebind<$lt>),+
            ) -> Out + Send + Sync + 'f,
        {
            fn from(f: Fun) -> Self {
                Self { inner: Arc::new(f) }
            }
        }

        impl<'f, $($arg: Arg,)+ Out: 'f, Fun>
            From<Fun> for FuncMutRef<'f, Lend<($($arg,)+)>, Out>
        where
            Fun: for<$($lt),+> FnMut(
                $(<$arg as Arg>::Rebind<$lt>),+
            ) -> Out + Send + 'f,
        {
            fn from(f: Fun) -> Self {
                Self { inner: Box::new(f) }
            }
        }

        impl<'f, $($arg: Arg,)+ Out: 'f, Fun>
            From<Fun> for FuncOnceRef<'f, Lend<($($arg,)+)>, Out>
        where
            Fun: for<$($lt),+> FnOnce(
                $(<$arg as Arg>::Rebind<$lt>),+
            ) -> Out + Send + 'f,
        {
            fn from(f: Fun) -> Self {
                Self { inner: Box::new(f) }
            }
        }
    };
}

impl_from_tuple!();
impl_from_tuple!(A);
impl_from_tuple!(A, B);
impl_from_tuple!(A, B, C);
impl_from_tuple!(A, B, C, D);
impl_from_tuple!(A, B, C, D, E);
impl_from_tuple!(A, B, C, D, E, Func);
impl_from_tuple!(A, B, C, D, E, Func, G);
impl_from_tuple!(A, B, C, D, E, Func, G, H);

impl_from_lend!(A: 'x0);
impl_from_lend!(A: 'x0, B: 'x1);
impl_from_lend!(A: 'x0, B: 'x1, C: 'x2);
impl_from_lend!(A: 'x0, B: 'x1, C: 'x2, D: 'x3);
impl_from_lend!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4);
impl_from_lend!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, Func: 'x5);
impl_from_lend!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, Func: 'x5, G: 'x6);
impl_from_lend!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, Func: 'x5, G: 'x6, H: 'x7);
