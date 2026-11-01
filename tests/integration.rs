use closed::{Callable, CallableMut, CallableOnce, Future};
use std::sync::{Arc, Mutex};
use std::thread;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn test_callable_is_send_and_sync() {
    assert_send_sync::<Callable<(), ()>>();
    assert_send_sync::<Callable<(String, i32), bool>>();
}

#[test]
fn test_callable_across_threads() {
    let shared_state = Arc::new(Mutex::new(0));
    let state_clone = Arc::clone(&shared_state);

    let task = Callable::<(i32,), ()>::new(move |val| {
        let mut lock = state_clone.lock().unwrap();
        *lock += val;
    });

    let task_clone = task.clone();

    let handle1 = thread::spawn(move || {
        task_clone.invoke((10,));
    });

    let handle2 = thread::spawn(move || {
        task.invoke((20,));
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    assert_eq!(*shared_state.lock().unwrap(), 30);
}

#[test]
fn test_callable_mut_state() {
    let mut counter = 0;
    let mut f = CallableMut::<(), i32>::new(move || {
        counter += 1;
        counter
    });

    assert_eq!(f.invoke_mut(()), 1);
    assert_eq!(f.invoke_mut(()), 2);
    assert_eq!(f.invoke_mut(()), 3);
}

#[test]
fn test_callable_once_consumption() {
    let data = String::from("consumed");
    let data_len = data.len();

    let f = CallableOnce::<(), String>::new(move || data);

    let result = f.invoke_once(());
    assert_eq!(result, "consumed");
    assert_eq!(result.len(), data_len);
}

#[test]
fn test_custom_struct_callable() {
    fn my_func(x: i32) -> i32 {
        x * 100
    }

    let f: Callable<(i32,), i32> = Callable::new(my_func);
    assert_eq!(f.invoke((5,)), 500);
}

#[tokio::test]
async fn test_async_callable_execution() {
    let f: Callable<(u32,), Future<u32>> =
        Callable::new_async(|x| async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            x * 2
        });

    let result = f.invoke((21,)).await;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_async_callable_mut_execution() {
    let mut counter = 0;
    let mut f: CallableMut<(), Future<i32>> =
        CallableMut::new_async(move || async move {
            counter += 1;
            counter
        });

    let res = f.invoke_mut(()).await;
    assert_eq!(res, 1);
}

#[test]
fn test_callable_zero_args() {
    let f: Callable<(), String> = Callable::new(|| "no args".to_string());
    assert_eq!(f.invoke(()), "no args");
}

#[test]
fn test_callable_many_args() {
    let f: Callable<(i32, i32, i32, i32, i32, i32), i32> =
        Callable::new(|a, b, c, d, e, f| a + b + c + d + e + f);

    assert_eq!(f.invoke((1, 2, 3, 4, 5, 6)), 21);
}
