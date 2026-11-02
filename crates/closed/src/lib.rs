#![doc = include_str!("docs.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, missing_docs, clippy::empty_docs)]

/// Private types, traits and so on. Use it only if you known what are you doing.
#[cfg(feature = "private")]
pub mod private;

#[cfg(not(feature = "private"))]
mod private;

#[allow(unused_imports)]
use private::*;

#[doc(hidden)]
pub extern crate alloc;

extern crate self as closed;

/// Unstable public API. If there is something, it will be in stable public API
/// soon.
#[cfg(feature = "unstable")]
pub mod unstable {
    pub use crate::private::sig::Signature;
    // pub use crate::private::spec::{Reference, is_ref};
}

// --- public API --- //

pub use private::sig::{Arg, Lend, Mut, Own, Ref};
pub use private::types::{Never, Promise, PromisePin};
pub use private::wrap::{
    Func, FuncMut, FuncMutRef, FuncOnce, FuncOnceRef, FuncRef,
};

#[cfg(feature = "macros")]
pub use closed_proc_macros::{func, promise};
