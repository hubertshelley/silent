use criterion::{Criterion, criterion_group, criterion_main};
use silent::prelude::*;

fn simple_route_benchmark(c: &mut Criterion) {
    let route = Route::new("").get(|_req| async { Ok("hello world") });

    c.bench_function("simple route match", |b| {
        b.iter(|| {
            let req = Request::default();
            let _ = route.call(req);
        });
    });
}

fn nested_route_benchmark(c: &mut Criterion) {
    let route = Route::new("api/v1")
        .append(Route::new("users").get(|_req| async { Ok("users") }))
        .append(Route::new("posts").get(|_req| async { Ok("posts") }));

    c.bench_function("nested route match", |b| {
        b.iter(|| {
            let mut req = Request::default();
            *req.uri_mut() = "/api/v1/posts".parse().unwrap();
            let _ = route.call(req);
        });
    });
}

fn middleware_route_benchmark(c: &mut Criterion) {
    let route = Route::new("")
        .hook(silent::middlewares::RequestTimeLogger)
        .get(|_req| async { Ok("hello world") });

    c.bench_function("route with middleware", |b| {
        b.iter(|| {
            let req = Request::default();
            let _ = route.call(req);
        });
    });
}

fn complex_route_benchmark(c: &mut Criterion) {
    let route = Route::new("api/v1")
        .append(
            Route::new("users")
                .get(|_req| async { Ok("users") })
                .post(|_req| async { Ok("create user") }),
        )
        .append(
            Route::new("posts")
                .get(|_req| async { Ok("posts") })
                .post(|_req| async { Ok("create post") })
                .append(Route::new("comments").get(|_req| async { Ok("comments") })),
        );

    let mut group = c.benchmark_group("Complex Routes");
    group.bench_function("GET /api/v1/users", |b| {
        b.iter(|| {
            let mut req = Request::default();
            *req.uri_mut() = "/api/v1/users".parse().unwrap();
            let _ = route.call(req);
        });
    });
    group.bench_function("POST /api/v1/posts", |b| {
        b.iter(|| {
            let mut req = Request::default();
            *req.uri_mut() = "/api/v1/posts".parse().unwrap();
            *req.method_mut() = Method::POST;
            let _ = route.call(req);
        });
    });
    group.bench_function("GET /api/v1/posts/comments", |b| {
        b.iter(|| {
            let mut req = Request::default();
            *req.uri_mut() = "/api/v1/posts/comments".parse().unwrap();
            let _ = route.call(req);
        });
    });
    group.finish();
}

fn multiple_middleware_benchmark(c: &mut Criterion) {
    let route = Route::new("")
        .hook(silent::middlewares::RequestTimeLogger)
        .hook(silent::middlewares::RequestTimeLogger) // Using same middleware twice for testing
        .get(|_req| async { Ok("hello world") });

    c.bench_function("route with multiple middleware", |b| {
        b.iter(|| {
            let req = Request::default();
            let _ = route.call(req);
        });
    });
}

fn high_load_benchmark(c: &mut Criterion) {
    let route = Route::new("api/v1").append(
        Route::new("users")
            .get(|_req| async { Ok("users") })
            .post(|_req| async { Ok("create user") }),
    );

    let mut group = c.benchmark_group("High Load");
    group.sample_size(1000);
    group.bench_function("1000 sequential requests", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let mut req = Request::default();
                *req.uri_mut() = format!("/api/v1/users?page={}", i).parse().unwrap();
                let _ = route.call(req);
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    simple_route_benchmark,
    nested_route_benchmark,
    middleware_route_benchmark,
    complex_route_benchmark,
    multiple_middleware_benchmark,
    high_load_benchmark
);
criterion_main!(benches);
