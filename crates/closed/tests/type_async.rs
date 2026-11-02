//! Type Mode Tests - Async

use closed::{func, promise};

#[test]
fn test_type_mode_async() {
    let _: func!(i32 -> i32; async) = (|x| promise! { x * 2 }).into();
}

#[test]
fn test_type_mode_async_no_args() {
    let _: func!(-> i32; async) = (|| promise! { 42 }).into();
}

#[test]
fn test_type_mode_async_mut() {
    let mut counter = 0;
    let _: func!(-> i32; async mut) = (move || {
        counter += 1;
        let current = counter;
        promise! { current }
    })
    .into();
}

#[test]
fn test_type_mode_async_once() {
    let data = String::from("async consumed");
    let _: func!(-> String; async once) = (move || promise! { data }).into();
}
