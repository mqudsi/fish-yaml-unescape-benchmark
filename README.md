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
