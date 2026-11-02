//! Value Mode Tests - Async Closures

use closed::func;

#[tokio::test]
async fn test_value_mode_async() {
    let f: func!(i32 -> i32; async) = func!(async move |x: i32| x * 2);
    let result = f.invoke((5,)).await;
    assert_eq!(result, 10);
}

#[tokio::test]
async fn test_value_mode_async_no_args() {
    let f: func!(-> i32; async) = func!(async move || 42);
    let result = f.invoke(()).await;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_value_mode_async_multiple_args() {
    let f: func!(i32, i32 -> i32; async) =
        func!(async move |a: i32, b: i32| a + b);
    let result = f.invoke((3, 4)).await;
    assert_eq!(result, 7);
}

#[tokio::test]
async fn test_value_mode_async_with_work() {
    let f: func!(u32 -> u32; async) = func!(async move |x: u32| {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        x * 2
    });
    let result = f.invoke((21,)).await;
    assert_eq!(result, 42);
}
