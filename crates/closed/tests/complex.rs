//! Complex Scenarios

use closed::{Func, Promise, promise};

#[test]
fn test_func_in_struct() {
    struct Handler {
        callback: Func<(i32,), i32>,
    }

    let handler = Handler {
        callback: (|x| x * 2).into(),
    };

    assert_eq!(handler.callback.invoke((5,)), 10);
}

#[test]
fn test_func_as_parameter() {
    fn apply(f: &Func<(i32,), i32>, x: i32) -> i32 {
        f.invoke((x,))
    }

    let f: Func<(i32,), i32> = (|x| x + 1).into();
    assert_eq!(apply(&f, 5), 6);
}

#[test]
fn test_func_return() {
    fn make_adder(n: i32) -> Func<(i32,), i32> {
        (move |x| x + n).into()
    }

    let add5 = make_adder(5);
    assert_eq!(add5.invoke((10,)), 15);
}

#[tokio::test]
async fn test_async_func_in_struct() {
    struct AsyncHandler {
        callback: Func<(u32,), Promise<u32>>,
    }

    let handler = AsyncHandler {
        callback: (|x| promise! { x * 2 }).into(),
    };

    let result = handler.callback.invoke((21,)).await;
    assert_eq!(result, 42);
}
