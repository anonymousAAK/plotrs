use criterion::{criterion_group, criterion_main, Criterion};
use plotkit::prelude::*;
use plotkit::FigureExt;

fn bench_line_10k(c: &mut Criterion) {
    let x: Vec<f64> = (0..10_000).map(|i| i as f64 * 0.001).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    c.bench_function("line_10k_png", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.plot(&x, &y).unwrap();
            fig.to_png_bytes().unwrap()
        });
    });
}

fn bench_line_100k(c: &mut Criterion) {
    let x: Vec<f64> = (0..100_000).map(|i| i as f64 * 0.0001).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    c.bench_function("line_100k_png", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.plot(&x, &y).unwrap();
            fig.to_png_bytes().unwrap()
        });
    });
}

fn bench_scatter_10k(c: &mut Criterion) {
    let x: Vec<f64> = (0..10_000).map(|i| i as f64 * 0.001).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin() + 0.1).collect();

    c.bench_function("scatter_10k_png", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.scatter(&x, &y).unwrap();
            fig.to_png_bytes().unwrap()
        });
    });
}

fn bench_bar_100(c: &mut Criterion) {
    let cats: Vec<String> = (0..100).map(|i| format!("c{i}")).collect();
    let heights: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin().abs()).collect();

    c.bench_function("bar_100_png", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.bar(cats.as_slice(), &heights).unwrap();
            fig.to_png_bytes().unwrap()
        });
    });
}

fn bench_histogram_10k(c: &mut Criterion) {
    let data: Vec<f64> = (0..10_000).map(|i| (i as f64 * 0.001).sin()).collect();

    c.bench_function("histogram_10k_png", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.hist(&data, 50).unwrap();
            fig.to_png_bytes().unwrap()
        });
    });
}

fn bench_svg_line_10k(c: &mut Criterion) {
    let x: Vec<f64> = (0..10_000).map(|i| i as f64 * 0.001).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    c.bench_function("svg_line_10k", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.plot(&x, &y).unwrap();
            fig.to_svg_string().unwrap()
        });
    });
}

fn bench_figure_creation(c: &mut Criterion) {
    c.bench_function("figure_creation", |b| {
        b.iter(|| {
            let mut fig = Figure::new();
            let ax = fig.add_subplot(1, 1, 1);
            ax.set_title("Title");
            ax.set_xlabel("X");
            ax.set_ylabel("Y");
            fig
        });
    });
}

fn bench_tick_generation(c: &mut Criterion) {
    use plotkit::ticks::generate_ticks;

    c.bench_function("tick_generation", |b| {
        b.iter(|| {
            let ranges = [
                (0.0, 10.0),
                (0.0, 1.0),
                (-5.0, 5.0),
                (0.0, 1e6),
                (0.001, 0.01),
                (100.0, 200.0),
                (-1e3, 1e3),
                (0.0, 0.1),
            ];
            for (lo, hi) in &ranges {
                generate_ticks(*lo, *hi, 7, &plotkit::scale::Scale::Linear);
            }
        });
    });
}

fn bench_multi_subplot(c: &mut Criterion) {
    let x: Vec<f64> = (0..1_000).map(|i| i as f64 * 0.01).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    c.bench_function("multi_subplot_4x4_png", |b| {
        b.iter(|| {
            let mut fig = Figure::subplots(4, 4);
            for i in 0..16 {
                let ax = fig.axes_mut(i).unwrap();
                ax.plot(&x, &y).unwrap();
            }
            fig.to_png_bytes().unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_line_10k,
    bench_line_100k,
    bench_scatter_10k,
    bench_bar_100,
    bench_histogram_10k,
    bench_svg_line_10k,
    bench_figure_creation,
    bench_tick_generation,
    bench_multi_subplot,
);
criterion_main!(benches);
