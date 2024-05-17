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

    pub fn chunk_loop(s: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(s.len());
        let mut chars = s.iter().copied();
        loop {
            result.extend(chars.by_ref().take_while(|b| *b != b'\\'));
            match chars.next() {
                Some(b'\\') => result.push(b'\\'),
                Some(b'n') => result.push(b'\n'),
                Some(other) => {
                    // Unrecognized escape. This shouldn't happen, but just pass it through.
                    result.push(b'\\');
                    result.push(other);
                }
                None => {
                    break;
                }
            }
        }

        result
    }
}

#[test]
fn consistent_unescape() {
    use crate::find::find_fold;
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
        let result_splice = {
            let mut clone = Vec::new();
            clone.extend_from_slice(line.as_bytes());
            unescape::splice(&mut clone);
            clone
        };
        let result_char_loop = unescape::char_loop(line.as_bytes());
        let result_chunk_loop = unescape::chunk_loop(line.as_bytes());
        assert_eq!(
            from_utf8(&result_splice).unwrap(),
            from_utf8(&result_char_loop).unwrap(),
            "char_loop() does not match legacy behavior!\nOriginal line:\n{}", line
        );
        assert_eq!(
            from_utf8(&result_splice).unwrap(),
            from_utf8(&result_chunk_loop).unwrap(),
            "chunk_loop() does not match legacy behavior!\nOriginal line:\n{}", line
        );
    }
}
