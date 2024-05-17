# Benchmark Info

This is used to figure out how fish's unescape_yaml_fish_2_0() function that is
very much in the hot path should be optimized.

There are benchmarks for two different operations: a) determining if a line
needs to be unescaped at all, and b) actually unescaping the line in question.

The theory for the first is that if we use bytes.fold(..) instead of
bytes.contains(..), the lack of early termination will allow the compiler to
perform SIMD transformations and speed up the process (since most lines are
short, we are hoping that the wins from batch SIMD processing will win out over
the branchiness of early termination).

The theory for the second is that if a line contains a `\`, it is on average
more likely than not that it also contains another and so we should not use
Vec::splice(..) because we'll be overwriting what we just wrote shortly
thereafter. A byte-by-byte loop and a chunked approach are both offered up as
alternatives.

# Results

## `find_escape` benchmark

Compiled for `x86-64-v1`:

```
> env RUSTFLAGS="-C target-cpu=x86-64" cargo bench --bench find_escape

find_escape/slice.contains()/1000
                        time:   [11.915 µs 11.922 µs 11.928 µs]
find_escape/slice.iter().fold()/1000
                        time:   [7.6625 µs 7.6671 µs 7.6718 µs]
```

Compiled for `x86-64-v3`:

```
> env RUSTFLAGS="-C target-cpu=x86-64-v3" cargo bench --bench find_escape

find_escape/slice.contains()/1000
                        time:   [10.949 µs 10.958 µs 10.968 µs]
find_escape/slice.iter().fold()/1000
                        time:   [9.5268 µs 9.5341 µs 9.5420 µs]
```

Compiled for `znver1`:

```
> env RUSTFLAGS="-C target-cpu=znver1" cargo bench --bench find_escape

find_escape/slice.contains()/1000
                        time:   [11.106 µs 11.112 µs 11.119 µs]
find_escape/slice.iter().fold()/1000
                        time:   [9.6777 µs 9.6828 µs 9.6879 µs]
```

## `find_escape` benchmark

Compiled for `x86-64-v1`:

```
> env RUSTFLAGS="-C target-cpu=x86-64" cargo bench --bench yaml_unescape

find_escape/char_loop()/1000
                        time:   [211.61 µs 211.75 µs 211.91 µs]
find_escape/splice()/1000
                        time:   [169.58 µs 169.83 µs 170.09 µs]
find_escape/chunk_loop_vec()/1000
                        time:   [209.93 µs 210.04 µs 210.15 µs]
find_escape/chunk_loop_box()/1000
                        time:   [149.61 µs 149.69 µs 149.78 µs]
```

Compiled for `x86-64-v3`:

```
> env RUSTFLAGS="-C target-cpu=x86-64-v3" cargo bench --bench yaml_unescape

find_escape/char_loop()/1000
                        time:   [199.31 µs 199.41 µs 199.52 µs]
find_escape/splice()/1000
                        time:   [168.03 µs 168.14 µs 168.26 µs]
find_escape/chunk_loop_vec()/1000
                        time:   [195.19 µs 195.41 µs 195.61 µs]
find_escape/chunk_loop_box()/1000
                        time:   [149.85 µs 149.94 µs 150.04 µs]
```

Compiled for `znver1`:

```
> env RUSTFLAGS="-C target-cpu=znver1" cargo bench --bench yaml_unescape

find_escape/char_loop()/1000
                        time:   [199.30 µs 199.39 µs 199.49 µs]
find_escape/splice()/1000
                        time:   [169.51 µs 169.68 µs 169.87 µs]
find_escape/chunk_loop_vec()/1000
                        time:   [217.09 µs 217.19 µs 217.29 µs]
find_escape/chunk_loop_box()/1000
                        time:   [153.02 µs 153.23 µs 153.57 µs]
```
