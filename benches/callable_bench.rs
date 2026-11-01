use closed::{Callable, CallableMut, CallableOnce};
use criterion::{Criterion, criterion_group, criterion_main};
use std::{hint::black_box, sync::Arc};

fn bench_callable_invoke(c: &mut Criterion) {
    let mut group = c.benchmark_group("Callable_invoke_vs_Arc");

    let closed_0 = Callable::<(), i32>::new(|| 42);
    let arc_0: Arc<dyn Fn() -> i32 + Send + Sync> = Arc::new(|| 42);

    group.bench_function("closed_0_args", |b| {
        b.iter(|| black_box(closed_0.invoke(())))
    });
    group.bench_function("arc_0_args", |b| b.iter(|| black_box(arc_0())));

    let closed_1 = Callable::<(i32,), i32>::new(|x| x + 1);
    let arc_1: Arc<dyn Fn(i32) -> i32 + Send + Sync> = Arc::new(|x| x + 1);

    group.bench_function("closed_1_arg", |b| {
        b.iter(|| black_box(closed_1.invoke((black_box(42),))))
    });
    group.bench_function("arc_1_arg", |b| {
        b.iter(|| black_box(arc_1(black_box(42))))
    });

    let closed_2 = Callable::<(i32, i32), i32>::new(|a, b| a + b);
    let arc_2: Arc<dyn Fn(i32, i32) -> i32 + Send + Sync> =
        Arc::new(|a, b| a + b);

    group.bench_function("closed_2_args", |b| {
        b.iter(|| black_box(closed_2.invoke((black_box(40), black_box(2)))))
    });
    group.bench_function("arc_2_args", |b| {
        b.iter(|| black_box(arc_2(black_box(40), black_box(2))))
    });

    group.finish();
}

fn bench_callable_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("Callable_call_vs_Arc");

    let closed_0 = Callable::<(), i32>::new(|| 42);
    let arc_0: Arc<dyn Fn() -> i32 + Send + Sync> = Arc::new(|| 42);

    group.bench_function("closed_0_args", |b| {
        b.iter(|| black_box(closed_0.call()))
    });
    group.bench_function("arc_0_args", |b| b.iter(|| black_box(arc_0())));

    let closed_1 = Callable::<(i32,), i32>::new(|x| x + 1);
    let arc_1: Arc<dyn Fn(i32) -> i32 + Send + Sync> = Arc::new(|x| x + 1);

    group.bench_function("closed_1_arg", |b| {
        b.iter(|| black_box(closed_1.call(black_box(42))))
    });
    group.bench_function("arc_1_arg", |b| {
        b.iter(|| black_box(arc_1(black_box(42))))
    });

    let closed_2 = Callable::<(i32, i32), i32>::new(|a, b| a + b);
    let arc_2: Arc<dyn Fn(i32, i32) -> i32 + Send + Sync> =
        Arc::new(|a, b| a + b);

    group.bench_function("closed_2_args", |b| {
        b.iter(|| black_box(closed_2.call(black_box(40), black_box(2))))
    });
    group.bench_function("arc_2_args", |b| {
        b.iter(|| black_box(arc_2(black_box(40), black_box(2))))
    });

    group.finish();
}

fn bench_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clone_Cost");

    let closed_fn = Callable::<(i32,), i32>::new(|x| x * 2);
    let arc_fn: Arc<dyn Fn(i32) -> i32 + Send + Sync> = Arc::new(|x| x * 2);

    group.bench_function("closed_clone", |b| {
        b.iter(|| {
            let c = closed_fn.clone();
            black_box(c);
        })
    });

    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            let a = Arc::clone(&arc_fn);
            black_box(a);
        })
    });

    group.finish();
}

fn bench_mut_and_once_invoke(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mut_and_Once_invoke_vs_Box");

    let mut count = 0;
    let mut closed_mut = CallableMut::<(), i32>::new(move || {
        count += 1;
        count
    });
    let mut box_mut: Box<dyn FnMut() -> i32 + Send> = Box::new({
        let mut count = 0;
        move || {
            count += 1;
            count
        }
    });

    group.bench_function("closed_mut", |b| {
        b.iter(|| black_box(closed_mut.invoke_mut(())))
    });
    group.bench_function("box_mut", |b| b.iter(|| black_box(box_mut())));

    group.bench_function("closed_once_zst_construct_and_call", |b| {
        b.iter(|| {
            let c = CallableOnce::<(), i32>::new(|| 42);
            black_box(c.invoke_once(()))
        })
    });
    group.bench_function("box_once_zst_construct_and_call", |b| {
        b.iter(|| {
            let b: Box<dyn FnOnce() -> i32 + Send> = Box::new(|| 42);
            black_box(b())
        })
    });

    group.finish();
}

fn bench_mut_and_once_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mut_and_Once_call_vs_Box");

    let mut count = 0;
    let mut closed_mut = CallableMut::<(), i32>::new(move || {
        count += 1;
        count
    });
    let mut box_mut: Box<dyn FnMut() -> i32 + Send> = Box::new({
        let mut count = 0;
        move || {
            count += 1;
            count
        }
    });

    group.bench_function("closed_mut", |b| {
        b.iter(|| black_box(closed_mut.call_mut()))
    });
    group.bench_function("box_mut", |b| b.iter(|| black_box(box_mut())));

    group.bench_function("closed_once_zst_construct_and_call", |b| {
        b.iter(|| {
            let c = CallableOnce::<(), i32>::new(|| 42);
            black_box(c.call_once())
        })
    });
    group.bench_function("box_once_zst_construct_and_call", |b| {
        b.iter(|| {
            let b: Box<dyn FnOnce() -> i32 + Send> = Box::new(|| 42);
            black_box(b())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_callable_invoke,
    bench_callable_call,
    bench_clone,
    bench_mut_and_once_invoke,
    bench_mut_and_once_call
);
criterion_main!(benches);
