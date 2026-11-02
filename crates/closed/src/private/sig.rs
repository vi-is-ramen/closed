//! Signature and FnArgs traits.

use core::marker::PhantomData;

/// A trait that extracts the output type from a callable.
pub trait Signature<Args> {
    /// Callable's output type.
    type Out;
}

macro_rules! impl_signature {
    ($($arg:ident),*) => {
        impl<Func, $($arg,)* Out> Signature<($($arg,)*)> for Func
        where
            Func: Fn($($arg),*) -> Out,
        {
            type Out = Out;
        }
    };
}

impl_signature!();
impl_signature!(A);
impl_signature!(A, B);
impl_signature!(A, B, C);
impl_signature!(A, B, C, D);
impl_signature!(A, B, C, D, E);
impl_signature!(A, B, C, D, E, F);
impl_signature!(A, B, C, D, E, F, G);
impl_signature!(A, B, C, D, E, F, G, H);

/// Maps a tuple of arguments to the corresponding `dyn Fn*` trait objects,
/// parameterized by a lifetime.
pub trait FnArgs {
    /// [`Fn`](core::ops::Fn) version of signature.
    type DynFn<'a, Out: 'a>: ?Sized + Send + Sync + 'a
    where
        Self: 'a;

    /// [`FnMut`](core::ops::FnMut) version of signature.
    type DynFnMut<'a, Out: 'a>: ?Sized + Send + 'a
    where
        Self: 'a;

    /// [`FnOnce`](core::ops::FnOnce) version of signature.
    type DynFnOnce<'a, Out: 'a>: ?Sized + Send + 'a
    where
        Self: 'a;
}

macro_rules! impl_fn_args {
    ($($arg:ident),*) => {
        impl<$($arg,)*> FnArgs for ($($arg,)*) {
            type DynFn<'a, Out: 'a> = dyn Fn($($arg),*) -> Out + Send + Sync + 'a
            where
                Self: 'a,
                Out: 'a;

            type DynFnMut<'a, Out: 'a> = dyn FnMut($($arg),*) -> Out + Send + 'a
            where
                Self: 'a,
                Out: 'a;

            type DynFnOnce<'a, Out: 'a> = dyn FnOnce($($arg),*) -> Out + Send + 'a
            where
                Self: 'a,
                Out: 'a;
        }
    };
}

impl_fn_args!();
impl_fn_args!(A);
impl_fn_args!(A, B);
impl_fn_args!(A, B, C);
impl_fn_args!(A, B, C, D);
impl_fn_args!(A, B, C, D, E);
impl_fn_args!(A, B, C, D, E, F);
impl_fn_args!(A, B, C, D, E, F, G);
impl_fn_args!(A, B, C, D, E, F, G, H);

// --- Arg marker types ---

/// Marker: argument is `&'x T` for any `'x`.
pub struct Ref<T: ?Sized>(PhantomData<fn() -> *const T>);
/// Marker: argument is `&'x mut T` for any `'x`.
pub struct Mut<T: ?Sized>(PhantomData<fn() -> *const T>);
/// Marker: argument is owned `T` (`T: 'static`).
pub struct Own<T: 'static>(PhantomData<fn() -> T>);

impl<T: ?Sized> core::fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Ref")
    }
}

impl<T: ?Sized> core::fmt::Debug for Mut<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Mut")
    }
}

impl<T: 'static> core::fmt::Debug for Own<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Own")
    }
}

/// General argument marker trait.
pub trait Arg {
    /// Rebind method marker.
    type Rebind<'x>
    where
        Self: 'x;
}

impl<T: ?Sized> Arg for Ref<T> {
    // `Self: 'x` ⇒ `*const T: 'x` ⇒ `T: 'x`, что нужно для `&'x T`.
    type Rebind<'x>
        = &'x T
    where
        Self: 'x;
}

impl<T: ?Sized> Arg for Mut<T> {
    type Rebind<'x>
        = &'x mut T
    where
        Self: 'x;
}

impl<T: 'static> Arg for Own<T> {
    // `T: 'static` ⇒ `T: 'x` для любого `'x`, значит where-clause всегда
    // выполняется и `Rebind<'x> = T` корректен.
    type Rebind<'x>
        = T
    where
        Self: 'x;
}

/// HRTB lending adapter.
pub struct Lend<T>(PhantomData<T>);

impl<T> core::fmt::Debug for Lend<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Lend")
    }
}

macro_rules! impl_lend_fn_args {
    ($($arg:ident : $lt:lifetime),+) => {
        impl<$($arg: Arg,)+> FnArgs for Lend<($($arg,)+)> {
            type DynFn<'f, Out: 'f> =
                dyn for<$($lt),+> Fn(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Out + Send + Sync + 'f
            where
                Self: 'f,
                Out: 'f;

            type DynFnMut<'f, Out: 'f> =
                dyn for<$($lt),+> FnMut(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Out + Send + 'f
            where
                Self: 'f,
                Out: 'f;

            type DynFnOnce<'f, Out: 'f> =
                dyn for<$($lt),+> FnOnce(
                    $(<$arg as Arg>::Rebind<$lt>),+
                ) -> Out + Send + 'f
            where
                Self: 'f,
                Out: 'f;
        }
    };
}

impl_lend_fn_args!(A: 'x0);
impl_lend_fn_args!(A: 'x0, B: 'x1);
impl_lend_fn_args!(A: 'x0, B: 'x1, C: 'x2);
impl_lend_fn_args!(A: 'x0, B: 'x1, C: 'x2, D: 'x3);
impl_lend_fn_args!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4);
impl_lend_fn_args!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, F: 'x5);
impl_lend_fn_args!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, F: 'x5, G: 'x6);
impl_lend_fn_args!(A: 'x0, B: 'x1, C: 'x2, D: 'x3, E: 'x4, F: 'x5, G: 'x6, H: 'x7);
