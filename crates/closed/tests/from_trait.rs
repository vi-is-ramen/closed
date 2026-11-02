//! From Trait Tests (.into())

use closed::{Func, FuncMut, FuncOnce};

#[test]
fn test_into_func() {
    let f: Func<(i32,), i32> = (|x: i32| x + 1).into();
    assert_eq!(f.invoke((5,)), 6);
}

#[test]
fn test_into_func_mut() {
    let mut counter = 0;
    let f: FuncMut<(), i32> = (move || {
        counter += 1;
        counter
    })
    .into();

    let mut f = f;
    assert_eq!(f.invoke_mut(()), 1);
}

#[test]
fn test_into_func_once() {
    let data = String::from("test");
    let f: FuncOnce<(), String> = (move || data).into();

    assert_eq!(f.invoke_once(()), "test");
}
