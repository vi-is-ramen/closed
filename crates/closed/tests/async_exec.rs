//! Async Execution Tests

use closed::{func, promise};

#[tokio::test]
async fn test_async_execution() {
    let f: func!(u32 -> u32; async) = (|x| {
        promise! {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            x * 2
        }
    })
    .into();

    let result = f.invoke((21,)).await;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_async_mut_execution() {
    let mut counter = 0;
    let mut f: func!(-> i32; async mut) = (move || {
        // 1. mutate state while we are sync (FnMut)
        counter += 1;
        let current = counter;

        // 2. promise to return the value
        promise! { current }
    })
    .into();

    assert_eq!(f.invoke_mut(()).await, 1);
    assert_eq!(f.invoke_mut(()).await, 2);
    assert_eq!(f.invoke_mut(()).await, 3);
}

#[tokio::test]
async fn test_async_once_execution() {
    let data = String::from("async once");
    let f: func!(-> String; async once) = (move || promise! { data }).into();

    let result = f.invoke_once(()).await;
    assert_eq!(result, "async once");
}
