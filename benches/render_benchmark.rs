use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rsdrav::prelude::*;
use rsdrav::render::{Buffer, Cell};

fn bench_buffer_creation(c: &mut Criterion) {
    c.bench_function("buffer_new_80x24", |b| {
        b.iter(|| Buffer::new(black_box(80), black_box(24)))
    });

    c.bench_function("buffer_new_120x40", |b| {
        b.iter(|| Buffer::new(black_box(120), black_box(40)))
    });
}

fn bench_buffer_operations(c: &mut Criterion) {
    let mut buf = Buffer::new(80, 24);
    let cell = Cell::new('X');

    c.bench_function("buffer_set_cell", |b| {
        b.iter(|| buf.set(black_box(40), black_box(12), black_box(cell.clone())))
    });

    c.bench_function("buffer_clear", |b| b.iter(|| buf.clear()));
}

fn bench_diff_algorithm(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_algorithm");

    for size in [24, 40, 60].iter() {
        let mut old_buf = Buffer::new(80, *size);
        let mut new_buf = Buffer::new(80, *size);

        // Fill with different content
        for y in 0..*size {
            for x in 0..80 {
                old_buf.set(x, y, Cell::new('A'));
                new_buf.set(x, y, Cell::new('B'));
            }
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                // Simulate diff operation by comparing buffers
                let mut changes = 0;
                for y in 0..*size {
                    let old_line = old_buf.line(y);
                    let new_line = new_buf.line(y);
                    if old_line != new_line {
                        changes += 1;
                    }
                }
                black_box(changes)
            })
        });
    }
    group.finish();
}

fn bench_signal_operations(c: &mut Criterion) {
    c.bench_function("signal_create", |b| b.iter(|| Signal::new(black_box(42))));

    let sig = Signal::new(0);
    c.bench_function("signal_get", |b| b.iter(|| sig.get()));

    let sig = Signal::new(0);
    c.bench_function("signal_set", |b| b.iter(|| sig.set(black_box(42))));

    let sig = Signal::new(0);
    c.bench_function("signal_update", |b| b.iter(|| sig.update(|v| *v += 1)));
}

fn bench_component_render(c: &mut Criterion) {
    let mut buf = Buffer::new(80, 24);
    let store = Store::new();
    let area = Rect::new(0, 0, 80, 24);

    c.bench_function("text_render", |b| {
        let text = Text::new("Hello, World!");
        b.iter(|| {
            let ctx = RenderContext::new(&mut buf, area, &store);
            black_box(text.render(&ctx))
        })
    });

    c.bench_function("vstack_render", |b| {
        let stack = VStack::new()
            .push(Text::new("Line 1"))
            .push(Text::new("Line 2"))
            .push(Text::new("Line 3"));

        b.iter(|| {
            let ctx = RenderContext::new(&mut buf, area, &store);
            black_box(stack.render(&ctx))
        })
    });
}

fn bench_layout_calculations(c: &mut Criterion) {
    c.bench_function("rect_operations", |b| {
        let rect = Rect::new(0, 0, 80, 24);
        b.iter(|| {
            let inner = rect.inner(black_box(1));
            let split = inner.split_h(black_box(50));
            black_box(split)
        })
    });

    c.bench_function("row_layout", |b| {
        let area = Rect::new(0, 0, 80, 24);
        let row = Row::new();
        let widths = vec![Length::Fixed(20), Length::Fill(1), Length::Fixed(20)];
        b.iter(|| {
            let rects = row.layout(black_box(area), black_box(&widths));
            black_box(rects)
        })
    });
}

criterion_group!(
    benches,
    bench_buffer_creation,
    bench_buffer_operations,
    bench_diff_algorithm,
    bench_signal_operations,
    bench_component_render,
    bench_layout_calculations
);
criterion_main!(benches);
