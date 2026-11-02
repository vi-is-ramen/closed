//! Invocation Methods Tests

use closed::func;

#[test]
fn test_invoke() {
    let f: func!(i32, i32 -> i32) = (|a, b| a + b).into();

    assert_eq!(f.invoke((3, 4)), 7);
}

#[test]
fn test_invoke_mut() {
    let mut counter = 0;
    let mut f: func!(i32 -> i32; mut) = (move |x| {
        counter += x;
        counter
    })
    .into();

    assert_eq!(f.invoke_mut((5,)), 5);
}

#[test]
fn test_invoke_once() {
    let data = [1, 2, 3];
    let f: func!(usize -> i32; once) = (move |idx| data[idx]).into();

    assert_eq!(f.invoke_once((1,)), 2);
}
