//! Type Mode Tests - Lifetimes

use closed::func;

#[test]
fn test_type_mode_with_ref() {
    let _: func!(&str -> usize) = (|s: &str| s.len()).into();
}

#[test]
fn test_type_mode_with_ref_mut() {
    let _: func!(&mut Vec<u8> -> usize; mut) =
        (|v: &mut Vec<u8>| v.len()).into();
}

#[test]
fn test_type_mode_with_ref_once() {
    let _: func!(&str -> usize; once) = (|s: &str| s.len()).into();
}
