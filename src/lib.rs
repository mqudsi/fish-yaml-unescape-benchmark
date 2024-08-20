pub mod find {
    #[inline]
    pub fn find_contains(slice: &[u8]) -> bool {
        slice.contains(&b'\\')
    }

    #[inline]
    pub fn find_fold(slice: &[u8]) -> bool {
        slice
            .into_iter()
            .copied()
            .fold(false, |acc, b| acc | (b == b'\\'))
    }
}

pub mod unescape {
    pub fn char_loop(s: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(s.len());
        let mut chars = s.iter().copied();
        loop {
            match chars.next() {
                Some(b'\\') => match chars.next() {
                    Some(b'\\') => result.push(b'\\'),
                    Some(b'n') => result.push(b'\n'),
                    Some(other) => {
                        // Unrecognized escape. Pass it through.
                        result.push(b'\\');
                        result.push(other);
                    }
                    // We don't expect a dangling escape, but pass it through.
                    None => result.push(b'\\'),
                },
                Some(other) => result.push(other),
                None => break,
            }
        }

        result
    }

    pub fn splice(s: &mut Vec<u8>) {
        let mut cursor = 0;
        while cursor < s.len() {
            // Look for a backslash.
            let Some(backslash) = s[cursor..].iter().position(|&c| c == b'\\') else {
                // No more backslashes
                break;
            };

            // Add back the start offset
            let backslash = backslash + cursor;

            // Backslash found. Maybe we'll do something about it.
            let Some(escaped_char) = s.get(backslash + 1) else {
                // Backslash was final character
                break;
            };

            match escaped_char {
                b'\\' => {
                    // Two backslashes in a row. Delete the second one.
                    s.remove(backslash + 1);
                }
                b'n' => {
                    // Backslash + n. Replace with a newline.
                    s.splice(backslash..(backslash + 2), [b'\n']);
                }
                _ => {
                    // Unknown backslash escape, keep as-is
                }
            };

            // The character at index backslash has now been made whole; start at the next
            // character.
            cursor = backslash + 1;
        }
    }

    pub fn chunk_loop_vec(s: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(s.len());
        let mut chars = s.iter().copied();
        loop {
            result.extend(chars.by_ref().take_while(|b| *b != b'\\'));
            match chars.next() {
                Some(b'\\') => result.push(b'\\'),
                Some(b'n') => result.push(b'\n'),
                _ => break,
            }
        }

        result
    }

    /// Inspecting the generated asm from [`chunk_loop_vec()`], it's clear that it's a suboptimal
    /// solution because `Vec::extend(iter)` keeps checking to see if it needs to resize, so this is
    /// an attempt to write to a `Box<[u8]>` directly (using known valid offsets) then convert the
    /// result to a (truncated) `Vec`.
    pub fn chunk_loop_box(s: &[u8]) -> Vec<u8> {
        // This is a very long way around of writing `Box::new_uninit_slice(s.len())`, which
        // requires the unstablized nightly-only feature new_unit (#63291). It optimizes away.
        let mut result: Box<[_]> = std::iter::repeat_with(std::mem::MaybeUninit::uninit)
            .take(s.len())
            .collect();
        let mut chars = s.iter().copied();
        let mut src_idx = 0;
        let mut dst_idx = 0;
        loop {
            // While inspecting the asm reveals the compiler does not elide the bounds check from
            // the writes to `result`, benchmarking shows that using `result.get_unchecked_mut()`
            // everywhere does not result in a statistically significant improvement to the
            // performance of this function.
            let to_copy = chars.by_ref().take_while(|b| *b != b'\\').count();
            unsafe {
                let src = s[src_idx..].as_ptr();
                // Can use the following when feature(maybe_uninit_slice) is stabilized:
                // let dst = std::mem::MaybeUninit::slice_as_mut_ptr(&mut result[dst_idx..]);
                let dst = result[dst_idx..].as_mut_ptr().cast();
                std::ptr::copy_nonoverlapping(src, dst, to_copy);
            }
            dst_idx += to_copy;

            match chars.next() {
                Some(b'\\') => result[dst_idx].write(b'\\'),
                Some(b'n') => result[dst_idx].write(b'\n'),
                _ => break,
            };
            src_idx += to_copy + 2;
            dst_idx += 1;
        }

        let result = Box::leak(result);
        unsafe { Vec::from_raw_parts(result.as_mut_ptr().cast(), dst_idx, result.len()) }
    }
}

#[test]
fn consistent_unescape() {
    use crate::find::find_fold;
    use crate::unescape::*;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::str::from_utf8;

    #[allow(deprecated)]
    let mut history_path = std::env::home_dir().unwrap();
    history_path.push(".local/share/fish/fish_history");
    if !history_path.exists() {
        return;
    }

    let history_file = BufReader::new(File::open(history_path).unwrap());
    let lines = history_file
        .lines()
        .map_while(|l| l.ok())
        // Read only lines with a slash, because that's the only time we call the functions.
        .filter(|l| find_fold(l.as_bytes()))
        // Cap at a sane amount
        .take(500);

    for line in lines {
        let expected = {
            let mut clone = Vec::new();
            clone.extend_from_slice(line.as_bytes());
            unescape::splice(&mut clone);
            clone
        };

        for (alternate, name) in [
            (char_loop as fn(&[u8]) -> Vec<u8>, stringify!(char_loop)),
            (chunk_loop_vec, stringify!(chunk_loop_vec)),
            (chunk_loop_box, stringify!(chunk_loop_box)),
        ] {
            let result = alternate(line.as_bytes());

            if &expected != &result {
                panic!(
                    concat!(
                        "{} does not match legacy behavior!\n",
                        "Original line:\n{}\n",
                        "Expected:\n{}\n",
                        "Actual:\n{}"
                    ),
                    name,
                    line,
                    from_utf8(&expected).unwrap(),
                    from_utf8(&result).unwrap()
                );
            }
        }
    }
}
