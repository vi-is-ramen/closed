//! Macro Mode Combinations

use closed::{func, promise};

#[test]
fn test_all_type_mode_combinations() {
    // Basic
    let _: func!(i32 -> i32) = (|x| x).into();

    // With mut
    let _: func!(i32 -> i32; mut) = (|x| x).into();

    // With once
    let _: func!(i32 -> i32; once) = (|x| x).into();

    // With async
    let _: func!(i32 -> i32; async) = (|x| promise! { x }).into();

    // With async and mut
    let _: func!(i32 -> i32; async mut) = (|x| promise! { x }).into();

    // With async and once
    let _: func!(i32 -> i32; async once) = (|x| promise! { x }).into();
}

#[tokio::test]
async fn test_all_value_mode_combinations() {
    // Sync
    let f: func!(i32 -> i32) = func!(|x: i32| x);
    assert_eq!(f.invoke((5,)), 5);

    // Sync with move
    let data = 42;
    let f: func!(-> i32) = func!(move || data);
    assert_eq!(f.invoke(()), 42);

    // Async
    let f: func!(i32 -> i32; async) = func!(async move |x: i32| x);
    assert_eq!(f.invoke((5,)).await, 5);
}
