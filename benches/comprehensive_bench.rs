use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rsdrav::prelude::*;

/// Comprehensive benchmark suite for rsdrav

fn bench_component_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_rendering");

    for widget_count in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*widget_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(widget_count),
            widget_count,
            |b, &count| {
                let mut buffer = Buffer::new(120, 40);
                let store = Store::new();
                let area = Rect::new(0, 0, 120, 40);

                b.iter(|| {
                    let mut stack = VStack::new();
                    for i in 0..count {
                        stack = stack.push(Text::new(format!("Item {}", i)));
                    }
                    let ctx = RenderContext::new(&mut buffer, area, &store);
                    black_box(stack.render(&ctx))
                });
            },
        );
    }
    group.finish();
}

fn bench_signal_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_operations");

    // Signal creation
    group.bench_function("signal_create", |b| b.iter(|| Signal::new(black_box(42))));

    // Signal get
    let sig = Signal::new(42);
    group.bench_function("signal_get", |b| b.iter(|| black_box(sig.get())));

    // Signal set
    let sig = Signal::new(0);
    group.bench_function("signal_set", |b| b.iter(|| sig.set(black_box(42))));

    // Signal update
    let sig = Signal::new(0);
    group.bench_function("signal_update", |b| b.iter(|| sig.update(|v| *v += 1)));

    // Derived computation
    let a = Signal::new(2);
    let b = Signal::new(3);
    let sum = {
        let a = a.clone();
        let b = b.clone();
        Derived::new(move || a.get() + b.get())
    };

    group.bench_function("derived_get_cached", |b| b.iter(|| black_box(sum.get())));

    group.finish();
}

fn bench_layout_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout");

    // Row layout
    let area = Rect::new(0, 0, 1000, 100);
    group.bench_function("row_fixed", |b| {
        let row = Row::new();
        let widths = vec![Length::Fixed(100), Length::Fixed(200), Length::Fixed(300)];
        b.iter(|| {
            black_box(row.layout(black_box(area), black_box(&widths)))
        })
    });

    // Column layout
    group.bench_function("column_fill", |b| {
        let col = Column::new();
        let heights = vec![Length::Fill(1), Length::Fill(2), Length::Fill(1)];
        b.iter(|| {
            black_box(col.layout(black_box(area), black_box(&heights)))
        })
    });

    // Flex layout
    group.bench_function("flex_complex", |b| {
        b.iter(|| {
            let flex = Flex::new(FlexDirection::Row)
                .add(FlexItem::new().grow(1.0).min(100))
                .add(FlexItem::new().grow(2.0).max(500))
                .add(FlexItem::new().fixed(200));
            black_box(flex.calculate(area))
        })
    });

    group.finish();
}

fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_widget");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        let items: Vec<String> = (0..*size).map(|i| format!("Item {}", i)).collect();
        let items_signal = Signal::new(items);
        let selected = Signal::new(Some(0));

        group.bench_with_input(BenchmarkId::new("render", size), size, |b, _| {
            let list = List::new(items_signal.clone(), selected.clone()).visible_height(20);

            let mut buffer = Buffer::new(80, 24);
            let store = Store::new();
            let area = Rect::new(0, 0, 80, 24);
            let ctx = RenderContext::new(&mut buffer, area, &store);

            b.iter(|| black_box(list.render(&ctx)))
        });
    }

    group.finish();
}

fn bench_table_operations(c: &mut Criterion) {
    #[derive(Clone)]
    struct Row {
        name: String,
        value: i32,
    }

    let mut group = c.benchmark_group("table_widget");

    for size in [50, 200, 1000].iter() {
        let rows: Vec<Row> = (0..*size)
            .map(|i| Row {
                name: format!("Row {}", i),
                value: i,
            })
            .collect();

        let data = Signal::new(rows);
        let selected = Signal::new(Some(0));

        group.bench_with_input(BenchmarkId::new("render", size), size, |b, _| {
            let table = Table::new(data.clone(), selected.clone())
                .column(TableColumn::new("Name", 20).render(|r: &Row| r.name.clone()))
                .column(TableColumn::new("Value", 10).render(|r: &Row| r.value.to_string()))
                .visible_height(10);

            let mut buffer = Buffer::new(80, 24);
            let store = Store::new();
            let area = Rect::new(0, 0, 80, 24);
            let ctx = RenderContext::new(&mut buffer, area, &store);

            b.iter(|| black_box(table.render(&ctx)))
        });
    }

    group.finish();
}

fn bench_animation(c: &mut Criterion) {
    use std::time::Duration;

    let mut group = c.benchmark_group("animation");

    group.bench_function("tween_update", |b| {
        let mut tween = Tween::new(0.0_f32, 100.0, Duration::from_secs(1));
        let delta = Duration::from_millis(16); // ~60fps

        b.iter(|| {
            tween.update(black_box(delta));
            black_box(tween.value())
        })
    });

    group.bench_function("easing_linear", |b| {
        let easing = EasingFunction::Linear;
        b.iter(|| black_box(easing.apply(0.5)))
    });

    group.bench_function("easing_cubic", |b| {
        let easing = EasingFunction::EaseInOutCubic;
        b.iter(|| black_box(easing.apply(0.5)))
    });

    group.finish();
}

fn bench_focus_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("focus");

    // Focus navigation with many components
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("navigation", count), count, |b, &count| {
            let mut focus = FocusManager::new();

            for i in 0..count {
                focus.register(ComponentId::new(i), i, true);
            }

            b.iter(|| {
                focus.focus_next();
                black_box(focus.current())
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_component_rendering,
    bench_signal_operations,
    bench_layout_calculations,
    bench_list_operations,
    bench_table_operations,
    bench_animation,
    bench_focus_management,
);
criterion_main!(benches);
