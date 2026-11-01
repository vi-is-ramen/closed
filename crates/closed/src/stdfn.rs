//! Implementations for arities 0 through 12.

#![allow(non_snake_case)]

use crate::traits::{
    CallableArgs, IntoAsyncCallable, IntoAsyncCallableMut,
    IntoAsyncCallableOnce, IntoCallable, IntoCallableMut, IntoCallableOnce,
};
use crate::types::{Callable, CallableMut, CallableOnce, Future};
use alloc::{boxed::Box, sync::Arc};

macro_rules! impl_arity {
    ($($arg:ident),*) => {
        // 1. Map tuples to `dyn Fn*` types.
        impl<$($arg: 'static,)*> CallableArgs for ($($arg,)*) {
            type DynFn<Output: 'static> = dyn Fn($($arg),*) -> Output + Send + Sync;
            type DynFnMut<Output: 'static> = dyn FnMut($($arg),*) -> Output + Send;
            type DynFnOnce<Output: 'static> = dyn FnOnce($($arg),*) -> Output + Send;
        }

        // 2. IntoCallable (Fn)
        impl<Func, $($arg: 'static,)* Output: 'static> IntoCallable<($($arg,)*), Output> for Func
        where
            Func: Fn($($arg),*) -> Output + Send + Sync + 'static,
        {
            fn into_callable(self) -> Callable<($($arg,)*), Output> {
                Callable { inner: Arc::new(self) }
            }
        }

        // 3. IntoCallableMut (FnMut)
        impl<Func, $($arg: 'static,)* Output: 'static> IntoCallableMut<($($arg,)*), Output> for Func
        where
            Func: FnMut($($arg),*) -> Output + Send + 'static,
        {
            fn into_callable_mut(self) -> CallableMut<($($arg,)*), Output> {
                CallableMut { inner: Box::new(self) }
            }
        }

        // 4. IntoCallableOnce (FnOnce)
        impl<Func, $($arg: 'static,)* Output: 'static> IntoCallableOnce<($($arg,)*), Output> for Func
        where
            Func: FnOnce($($arg),*) -> Output + Send + 'static,
        {
            fn into_callable_once(self) -> CallableOnce<($($arg,)*), Output> {
                CallableOnce { inner: Box::new(self) }
            }
        }

        // --- Async Implementations ---

        // 5. IntoAsyncCallable
        impl<Func, Fut, $($arg: 'static,)* Output: 'static> IntoAsyncCallable<($($arg,)*), Output> for Func
        where
            Func: Fn($($arg),*) -> Fut + Send + Sync + 'static,
            Fut: core::future::Future<Output = Output> + Send + 'static,
        {
            fn into_async_callable(self) -> Callable<($($arg,)*), Future<Output>> {
                Callable {
                    inner: Arc::new(move |$($arg),*| -> Future<Output> {
                        Box::pin(self($($arg),*))
                    }),
                }
            }
        }

        // 6. IntoAsyncCallableMut
        impl<Func, Fut, $($arg: 'static,)* Output: 'static> IntoAsyncCallableMut<($($arg,)*), Output> for Func
        where
            Func: FnMut($($arg),*) -> Fut + Send + 'static,
            Fut: core::future::Future<Output = Output> + Send + 'static,
        {
            fn into_async_callable_mut(mut self) -> CallableMut<($($arg,)*), Future<Output>> {
                CallableMut {
                    inner: Box::new(move |$($arg),*| -> Future<Output> {
                        Box::pin(self($($arg),*))
                    }),
                }
            }
        }

        // 7. IntoAsyncCallableOnce
        impl<Func, Fut, $($arg: 'static,)* Output: 'static> IntoAsyncCallableOnce<($($arg,)*), Output> for Func
        where
            Func: FnOnce($($arg),*) -> Fut + Send + 'static,
            Fut: core::future::Future<Output = Output> + Send + 'static,
        {
            fn into_async_callable_once(self) -> CallableOnce<($($arg,)*), Future<Output>> {
                CallableOnce {
                    inner: Box::new(move |$($arg),*| -> Future<Output> {
                        Box::pin(self($($arg),*))
                    }),
                }
            }
        }

        // --- Invoke methods ---

        // 8. Callable
        impl<$($arg: 'static,)* Output: 'static> Callable<($($arg,)*), Output> {
            /// Invokes the underlying callable with the given argument tuple.
            #[allow(non_snake_case)]
            pub fn invoke(&self, ($($arg,)*): ($($arg,)*)) -> Output {
                (self.inner)($($arg),*)
            }
        }

        // 9. CallableMut
        impl<$($arg: 'static,)* Output: 'static> CallableMut<($($arg,)*), Output> {
            /// Invokes the underlying callable, mutating its state.
            #[allow(non_snake_case)]
            pub fn invoke_mut(&mut self, ($($arg,)*): ($($arg,)*)) -> Output {
                (self.inner)($($arg),*)
            }
        }

        // 10. CallableOnce
        impl<$($arg: 'static,)* Output: 'static> CallableOnce<($($arg,)*), Output> {
            /// Invokes the underlying callable, consuming it.
            #[allow(non_snake_case)]
            pub fn invoke_once(self, ($($arg,)*): ($($arg,)*)) -> Output {
                (self.inner)($($arg),*)
            }
        }
    };
}

impl_arity!();
impl_arity!(A);
impl_arity!(A, B);
impl_arity!(A, B, C);
impl_arity!(A, B, C, D);
impl_arity!(A, B, C, D, E);
impl_arity!(A, B, C, D, E, F);
impl_arity!(A, B, C, D, E, F, G);
impl_arity!(A, B, C, D, E, F, G, H);
impl_arity!(A, B, C, D, E, F, G, H, I);
impl_arity!(A, B, C, D, E, F, G, H, I, J);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L);
