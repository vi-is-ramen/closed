//! Value Mode Tests - Sync Closures

use closed::func;

#[test]
fn test_value_mode_sync() {
    let f: func!(i32 -> i32) = func!(|x: i32| x + 1);
    assert_eq!(f.invoke((5,)), 6);
}

#[test]
fn test_value_mode_sync_move() {
    let data = String::from("hello");
    let f: func!(-> String; once) = func!(move || data);
    assert_eq!(f.invoke_once(()), "hello");
}

#[test]
fn test_value_mode_sync_multiple_args() {
    let f: func!(i32, i32 -> i32) = func!(|a: i32, b: i32| a + b);
    assert_eq!(f.invoke((3, 4)), 7);
}

#[test]
fn test_value_mode_sync_no_return() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);

    let f: func!(bool) = func!(|b: bool| {
        called_clone.store(b, Ordering::SeqCst);
    });

    f.invoke((true,));
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_value_mode_sync_with_return_type() {
    let f: func!(i32 -> i32) = func!(|x: i32 -> i32| { x * 2 });
    assert_eq!(f.invoke((5,)), 10);
}
