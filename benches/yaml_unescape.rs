use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs::File;
use std::io::{BufRead, BufReader};
extern crate unescape_bench;
use unescape_bench::find::find_fold;
use unescape_bench::unescape::{char_loop, splice, chunk_loop};

pub fn find_escape(c: &mut Criterion) {
    #[allow(deprecated)]
    let mut history_path = std::env::home_dir().unwrap();
    history_path.push(".local/share/fish/fish_history");
    assert!(history_path.exists());
    let history_file = BufReader::new(File::open(history_path).unwrap());
    let lines: Vec<_> = history_file
        .lines()
        .map_while(|l| l.ok())
        // Read only lines with a slash, because that's the only time we call the functions.
        .filter(|l| find_fold(l.as_bytes()))
        // Eat the cost of reading the lines up front.
        .collect();

    let mut group = c.benchmark_group("find_escape");
    for i in [100, 1_000, 1_0000].iter() {
        group.bench_with_input(BenchmarkId::new("char_loop()", i), &lines, |b, lines| {
            b.iter(|| {
                for line in lines.iter().take(*i) {
                    black_box(char_loop(line.as_bytes()));
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("splice()", i), &lines, |b, lines| {
            b.iter(|| {
                for line in lines.iter().take(*i) {
                    // The code we have clones the vector before passing it as `&mut Vec<u8>`
                    let mut v = Vec::with_capacity(line.len());
                    v.extend_from_slice(line.as_bytes());
                    black_box(splice(&mut v));
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("chunk_loop()", i), &lines, |b, lines| {
            b.iter(|| {
                for line in lines.iter().take(*i) {
                    black_box(chunk_loop(line.as_bytes()));
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, find_escape);
criterion_main!(benches);
