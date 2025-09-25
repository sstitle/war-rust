use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use war_rust::ring_buffer::RingBuffer;

fn bench_ring_buffer_creation(c: &mut Criterion) {
    c.bench_function("ring_buffer_creation_small", |b| {
        b.iter(|| black_box(RingBuffer::<i32, 10>::new(0)))
    });

    c.bench_function("ring_buffer_creation_large", |b| {
        b.iter(|| black_box(RingBuffer::<i32, 1000>::new(0)))
    });
}

fn bench_ring_buffer_basic_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_basic");

    group.bench_function("push_back", |b| {
        b.iter_batched(
            || RingBuffer::<i32, 1000>::new(0),
            |mut rb| {
                for i in 0..100 {
                    rb.push_back(i);
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("push_front", |b| {
        b.iter_batched(
            || RingBuffer::<i32, 1000>::new(0),
            |mut rb| {
                for i in 0..100 {
                    rb.push_front(i);
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("pop_back", |b| {
        b.iter_batched(
            || {
                let mut rb = RingBuffer::<i32, 1000>::new(0);
                for i in 0..100 {
                    rb.push_back(i);
                }
                rb
            },
            |mut rb| {
                while !rb.is_empty() {
                    black_box(rb.pop_back());
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("pop_front", |b| {
        b.iter_batched(
            || {
                let mut rb = RingBuffer::<i32, 1000>::new(0);
                for i in 0..100 {
                    rb.push_back(i);
                }
                rb
            },
            |mut rb| {
                while !rb.is_empty() {
                    black_box(rb.pop_front());
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_ring_buffer_multiple_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_multiple");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("push_back_multiple", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let rb = RingBuffer::<i32, 1000>::new(0);
                        let data: Vec<i32> = (0..size).collect();
                        (rb, data)
                    },
                    |(mut rb, data)| {
                        rb.push_back_multiple(&data);
                        black_box(rb)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("push_front_multiple", size),
            size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let rb = RingBuffer::<i32, 1000>::new(0);
                        let data: Vec<i32> = (0..size).collect();
                        (rb, data)
                    },
                    |(mut rb, data)| {
                        rb.push_front_multiple(&data);
                        black_box(rb)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

fn bench_ring_buffer_wraparound(c: &mut Criterion) {
    c.bench_function("ring_buffer_wraparound_stress", |b| {
        b.iter_batched(
            || RingBuffer::<i32, 100>::new(0),
            |mut rb| {
                // Fill the buffer
                for i in 0..100 {
                    rb.push_back(i);
                }

                // Cause many wraparounds by continuous push/pop
                for i in 100..1000 {
                    rb.pop_front();
                    rb.push_back(i);
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_ring_buffer_mixed_ops(c: &mut Criterion) {
    c.bench_function("ring_buffer_mixed_operations", |b| {
        b.iter_batched(
            || RingBuffer::<i32, 500>::new(0),
            |mut rb| {
                // Simulate realistic card game usage patterns
                for round in 0..50 {
                    // Add cards to back (dealing)
                    rb.push_back(round * 2);
                    rb.push_back(round * 2 + 1);

                    // Draw cards (from back)
                    black_box(rb.pop_back());
                    black_box(rb.pop_back());

                    // Sometimes add won cards to front
                    if round % 3 == 0 {
                        rb.push_front(round + 1000);
                        rb.push_front(round + 2000);
                    }

                    // Check status
                    black_box(rb.len());
                    black_box(rb.is_empty());
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_ring_buffer_memory_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_memory_sizes");

    // Test different buffer sizes to show scaling
    group.bench_function("size_10", |b| {
        b.iter_batched(
            || RingBuffer::<u8, 10>::new(0),
            |mut rb| {
                for i in 0..10 {
                    rb.push_back(i);
                }
                for _ in 0..10 {
                    black_box(rb.pop_front());
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("size_52", |b| {
        b.iter_batched(
            || RingBuffer::<u8, 52>::new(0),
            |mut rb| {
                for i in 0..52 {
                    rb.push_back(i);
                }
                for _ in 0..52 {
                    black_box(rb.pop_front());
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("size_1000", |b| {
        b.iter_batched(
            || RingBuffer::<u8, 1000>::new(0),
            |mut rb| {
                for i in 0..100 {
                    // Only use part of the buffer
                    rb.push_back(i as u8);
                }
                for _ in 0..100 {
                    black_box(rb.pop_front());
                }
                black_box(rb)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ring_buffer_creation,
    bench_ring_buffer_basic_ops,
    bench_ring_buffer_multiple_ops,
    bench_ring_buffer_wraparound,
    bench_ring_buffer_mixed_ops,
    bench_ring_buffer_memory_sizes
);

criterion_main!(benches);
