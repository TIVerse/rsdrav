//! Performance benchmarks for rsdrav
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsdrav::command;
use rsdrav::prelude::*;
use rsdrav::render::{compute_diff, Buffer, Cell};

fn bench_buffer_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer");

    group.bench_function("create_80x24", |b| {
        b.iter(|| Buffer::new(black_box(80), black_box(24)));
    });

    group.bench_function("create_120x30", |b| {
        b.iter(|| Buffer::new(black_box(120), black_box(30)));
    });

    group.bench_function("clear", |b| {
        let mut buffer = Buffer::new(80, 24);
        b.iter(|| buffer.clear());
    });

    group.bench_function("set_cell", |b| {
        let mut buffer = Buffer::new(80, 24);
        let cell = Cell::new('X');
        b.iter(|| {
            buffer.set(black_box(10), black_box(10), cell.clone());
        });
    });

    group.finish();
}

fn bench_diff_algorithm(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff");

    // Benchmark unchanged buffers (best case - all lines hash equal)
    group.bench_function("unchanged_80x24", |b| {
        let buf1 = Buffer::new(80, 24);
        let buf2 = Buffer::new(80, 24);
        b.iter(|| compute_diff(black_box(&buf1), black_box(&buf2)));
    });

    // Benchmark single cell change
    group.bench_function("single_cell_change", |b| {
        let buf1 = Buffer::new(80, 24);
        let mut buf2 = Buffer::new(80, 24);
        buf2.set(40, 12, Cell::new('X'));
        b.iter(|| compute_diff(black_box(&buf1), black_box(&buf2)));
    });

    // Benchmark full redraw (worst case)
    group.bench_function("full_redraw", |b| {
        let buf1 = Buffer::new(80, 24);
        let mut buf2 = Buffer::new(80, 24);
        // Fill buffer with content
        for y in 0..24 {
            for x in 0..80 {
                buf2.set(x, y, Cell::new('█'));
            }
        }
        b.iter(|| compute_diff(black_box(&buf1), black_box(&buf2)));
    });

    // Benchmark 10% change (realistic case)
    group.bench_function("10_percent_change", |b| {
        let buf1 = Buffer::new(80, 24);
        let mut buf2 = Buffer::new(80, 24);
        // Change roughly 10% of cells
        for y in (0..24).step_by(3) {
            for x in (0..80).step_by(3) {
                buf2.set(x, y, Cell::new('X'));
            }
        }
        b.iter(|| compute_diff(black_box(&buf1), black_box(&buf2)));
    });

    group.finish();
}

fn bench_layout_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout");

    group.bench_function("row_fixed", |b| {
        let area = Rect::new(0, 0, 100, 50);
        let row = Row::new();
        let lengths = vec![Length::Fixed(20), Length::Fixed(30), Length::Fixed(50)];
        b.iter(|| row.layout(black_box(area), black_box(&lengths)));
    });

    group.bench_function("row_fill", |b| {
        let area = Rect::new(0, 0, 100, 50);
        let row = Row::new();
        let lengths = vec![Length::Fill(1), Length::Fill(2), Length::Fill(1)];
        b.iter(|| row.layout(black_box(area), black_box(&lengths)));
    });

    group.bench_function("column_mixed", |b| {
        let area = Rect::new(0, 0, 100, 50);
        let col = Column::new();
        let lengths = vec![
            Length::Fixed(10),
            Length::Fill(1),
            Length::Percent(0.2),
            Length::Fill(2),
        ];
        b.iter(|| col.layout(black_box(area), black_box(&lengths)));
    });

    group.finish();
}

fn bench_signal_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal");

    group.bench_function("create", |b| {
        b.iter(|| Signal::new(black_box(42)));
    });

    group.bench_function("get", |b| {
        let sig = Signal::new(42);
        b.iter(|| sig.get());
    });

    group.bench_function("set", |b| {
        let sig = Signal::new(0);
        let mut counter = 0;
        b.iter(|| {
            sig.set(black_box(counter));
            counter += 1;
        });
    });

    group.bench_function("update", |b| {
        let sig = Signal::new(0);
        b.iter(|| {
            sig.update(|v| *v = black_box(*v + 1));
        });
    });

    group.bench_function("subscribe", |b| {
        let sig = Signal::new(0);
        b.iter(|| {
            let _sub = sig.subscribe(|_| {});
        });
    });

    // Benchmark with active subscribers
    group.bench_function("set_with_subscribers", |b| {
        let sig = Signal::new(0);
        let _sub1 = sig.subscribe(|_| {});
        let _sub2 = sig.subscribe(|_| {});
        let _sub3 = sig.subscribe(|_| {});

        let mut counter = 0;
        b.iter(|| {
            sig.set(black_box(counter));
            counter += 1;
        });
    });

    group.finish();
}

fn bench_derived_signals(c: &mut Criterion) {
    let mut group = c.benchmark_group("derived");

    group.bench_function("create", |bencher| {
        let a = Signal::new(2);
        let b = Signal::new(3);
        bencher.iter(|| {
            let _derived = Derived::new({
                let a = a.clone();
                let b = b.clone();
                move || a.get() + b.get()
            });
        });
    });

    group.bench_function("get_cached", |bencher| {
        let a = Signal::new(2);
        let b = Signal::new(3);
        let derived = Derived::new({
            let a = a.clone();
            let b = b.clone();
            move || a.get() + b.get()
        });

        // Prime the cache
        let _ = derived.get();

        bencher.iter(|| {
            black_box(derived.get());
        });
    });

    group.bench_function("get_uncached", |bencher| {
        let a = Signal::new(2);
        let b = Signal::new(3);
        let derived = Derived::new({
            let a = a.clone();
            let b = b.clone();
            move || a.get() + b.get()
        });

        bencher.iter(|| {
            derived.invalidate();
            black_box(derived.get());
        });
    });

    group.finish();
}

fn bench_command_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("command");

    group.bench_function("parse_simple", |b| {
        b.iter(|| command::parse(black_box("quit")));
    });

    group.bench_function("parse_with_args", |b| {
        b.iter(|| command::parse(black_box("echo hello world")));
    });

    group.bench_function("parse_with_quotes", |b| {
        b.iter(|| command::parse(black_box(r#"set name "John Doe""#)));
    });

    group.bench_function("parse_complex", |b| {
        b.iter(|| {
            command::parse(black_box(
                r#"deploy production --force --config "path/to/config.yml""#,
            ))
        });
    });

    group.finish();
}

fn bench_animation(c: &mut Criterion) {
    use rsdrav::animation::Tween;
    use std::time::Duration;

    let mut group = c.benchmark_group("animation");

    group.bench_function("tween_update", |b| {
        let mut tween = Tween::new(0.0_f32, 100.0_f32, Duration::from_secs(1));
        let delta = Duration::from_millis(16);
        b.iter(|| {
            tween.update(black_box(delta));
        });
    });

    group.bench_function("tween_value", |b| {
        let tween = Tween::new(0.0_f32, 100.0_f32, Duration::from_secs(1));
        b.iter(|| {
            black_box(tween.value());
        });
    });

    group.bench_function("color_lerp", |b| {
        let c1 = Color::RED;
        let c2 = Color::BLUE;
        b.iter(|| {
            use rsdrav::animation::Animatable;
            black_box(c1.lerp(&c2, 0.5));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_buffer_operations,
    bench_diff_algorithm,
    bench_layout_system,
    bench_signal_operations,
    bench_derived_signals,
    bench_command_parsing,
    bench_animation
);
criterion_main!(benches);
