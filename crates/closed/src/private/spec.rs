//! Runtime reflection over function-like types.

// temporary
#![allow(missing_docs, missing_debug_implementations, dead_code)]

// quick explanation about what is going on here:
//
// there is WIP on universal runtime type reflection system which would be used
// to reflect on function and closure types. Main intended use case is
// microoptimizations based on known type properties.

use alloc::boxed::Box;

pub enum Family {
    Ptr {
        // references and pointers
        is_ref: bool,
        is_mut: bool,
        inner: Box<Type>,
    },
    Fn {
        // function pointers
        args: Box<[Type]>,
        out: Box<Type>,
    },
    Tuple {
        // tuples and unit types
        len: usize,
        inner: Box<[Type]>,
    },
    Slice {
        // slices ([T; N] and [T])
        len: Option<usize>,
        inner: Box<Type>,
    },
    Struct,
}

pub struct Type {
    id: u64,
    size: usize,
    align: usize,
    family: Family,
}

trait ReflectFamily {
    fn family() -> Family;
}

trait Reflect: ReflectFamily {
    fn reflect() -> Type;
}

impl<T: ReflectFamily> Reflect for T {
    fn reflect() -> Type {
        let id;

        {
            let mut hasher = nohash::NoHashHasher::<u64>::default();
            core::hash::Hash::hash(&typeid::of::<T>(), &mut hasher);
            id = core::hash::Hasher::finish(&hasher);
        };

        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();
        let family = <T as ReflectFamily>::family();

        Type {
            id,
            size,
            align,
            family,
        }
    }
}
