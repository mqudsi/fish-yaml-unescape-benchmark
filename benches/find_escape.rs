use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs::File;
use std::io::{BufRead, BufReader};
extern crate unescape_bench;
use unescape_bench::find::{find_contains, find_fold};

pub fn find_escape(c: &mut Criterion) {
    #[allow(deprecated)]
    let mut history_path = std::env::home_dir().unwrap();
    history_path.push(".local/share/fish/fish_history");
    assert!(history_path.exists());
    let history_file = BufReader::new(File::open(history_path).unwrap());
    // Eat the cost of reading the lines up front.
    let lines: Vec<_> = history_file.lines().map_while(|l| l.ok()).collect();

    let mut group = c.benchmark_group("find_escape");
    for i in [1_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("slice.contains()", i),
            &lines,
            |b, lines| {
                b.iter(|| {
                    for line in lines.iter().take(*i) {
                        black_box(find_contains(line.as_bytes()));
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("slice.iter().fold()", i),
            &lines,
            |b, lines| {
                b.iter(|| {
                    for line in lines.iter().take(*i) {
                        black_box(find_fold(line.as_bytes()));
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, find_escape);
criterion_main!(benches);
