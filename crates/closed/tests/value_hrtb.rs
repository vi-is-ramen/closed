//! Value Mode Tests - HRTB (References)

use closed::func;

#[test]
fn test_value_mode_ref() {
    let f: func!(&str -> usize) = func!(|s: &str| s.len());
    assert_eq!(f.invoke(("hello",)), 5);
}

#[test]
fn test_value_mode_mut_ref() {
    let f: func!(&mut Vec<i32> -> usize) = func!(|v: &mut Vec<i32>| {
        v.push(42);
        v.len()
    });

    let mut vec = vec![1, 2, 3];
    assert_eq!(f.invoke((&mut vec,)), 4);
    assert_eq!(vec, vec![1, 2, 3, 42]);
}

#[test]
fn test_value_mode_multiple_refs() {
    let f: func!(&str, &mut Vec<u8> -> usize) =
        func!(|s: &str, v: &mut Vec<u8>| {
            v.extend_from_slice(s.as_bytes());
            v.len()
        });

    let mut buf = Vec::new();
    assert_eq!(f.invoke(("hi", &mut buf)), 2);
    assert_eq!(buf, b"hi");
}
