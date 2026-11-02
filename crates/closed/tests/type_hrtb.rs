//! Type Mode Tests - HRTB (References)

use closed::{func, promise};

#[test]
fn test_type_mode_ref() {
    let _: func!(&str -> usize) = (|s: &str| s.len()).into();
}

#[test]
fn test_type_mode_mut_ref() {
    let _: func!(&mut Vec<u8> -> usize) = (|v: &mut Vec<u8>| v.len()).into();
}

#[test]
fn test_type_mode_multiple_refs() {
    let _: func!(&str, &mut Vec<u8> -> usize) = (|s: &str, v: &mut Vec<u8>| {
        v.extend_from_slice(s.as_bytes());
        v.len()
    })
    .into();
}

#[test]
fn test_type_mode_ref_with_lifetime() {
    let _: func!(&str -> usize) = (|s: &str| s.len()).into();
}

#[test]
fn test_type_mode_ref_async() {
    // Note: async with HRTB requires owned data or copying before async block
    let _: func!(&str -> usize; async) = (|s: &str| {
        let len = s.len();
        promise! { len }
    })
    .into();
}
