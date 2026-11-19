use anyhow::Result;

use crate::cache;

/// "3" → Some((3, 3)), "2-4" → Some((2, 4)), anything else → None
fn parse_range(arg: &str) -> Option<(usize, usize)> {
    // Anything other than digits and hyphens is not a number
    if arg.is_empty() || !arg.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return None;
    }

    match arg.split_once('-') {
        Some((start, end)) => {
            let start: usize = start.parse().ok()?;
            let end: usize = end.parse().ok()?;
            if start <= end {
                Some((start, end))
            } else {
                None
            }
        }
        None => {
            let n: usize = arg.parse().ok()?;
            Some((n, n))
        }
    }
}

/// "1,3-4" → Some(vec![(1, 1), (3, 4)])
/// Parses each comma-separated part with parse_range; None if any part is not a number
fn parse_ranges(arg: &str) -> Option<Vec<(usize, usize)>> {
    arg.split(',').map(parse_range).collect()
}

/// Replaces numeric arguments with file paths
/// Out-of-range numbers and non-numeric arguments are kept as they are
pub fn expand(args: &[String], files: &[String]) -> Vec<String> {
    let mut result = Vec::new();

    for arg in args {
        match parse_ranges(arg) {
            // Expand only when every range fits within the number of files
            Some(ranges)
                if ranges
                    .iter()
                    .all(|&(start, end)| start >= 1 && end <= files.len()) =>
            {
                for (start, end) in ranges {
                    result.extend_from_slice(&files[start - 1..end]);
                }
            }
            _ => result.push(arg.clone()),
        }
    }

    result
}

/// Loads the cache and expands the arguments if any of them is a number
pub fn expand_with_cache(args: &[String]) -> Result<Vec<String>> {
    let has_number = args.iter().any(|arg| parse_ranges(arg).is_some());
    if !has_number {
        return Ok(args.to_vec());
    }

    let files = cache::load()?;
    Ok(expand(args, &files))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Converts an array of &str into Vec<String> for tests
    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_single_number_and_range() {
        assert_eq!(parse_range("3"), Some((3, 3)));
        assert_eq!(parse_range("2-4"), Some((2, 4)));
    }

    #[test]
    fn rejects_non_numbers() {
        assert_eq!(parse_range("src/main.rs"), None);
        assert_eq!(parse_range("--cached"), None);
        assert_eq!(parse_range("+3"), None);
        assert_eq!(parse_range("4-2"), None);
        assert_eq!(parse_range("1-2-3"), None);
        assert_eq!(parse_range(""), None);
    }

    #[test]
    fn expands_numbers_and_ranges() {
        let files = strings(&["a.rs", "b.rs", "c.rs", "d.rs"]);
        let args = strings(&["--cached", "1", "3-4"]);
        assert_eq!(
            expand(&args, &files),
            strings(&["--cached", "a.rs", "c.rs", "d.rs"])
        );
    }

    #[test]
    fn expands_comma_separated_numbers() {
        let files = strings(&["a.rs", "b.rs", "c.rs", "d.rs"]);
        let args = strings(&["--cached", "1,3,4"]);
        assert_eq!(
            expand(&args, &files),
            strings(&["--cached", "a.rs", "c.rs", "d.rs"])
        );
    }

    #[test]
    fn parses_comma_separated_numbers_and_ranges() {
        assert_eq!(parse_ranges("3"), Some(vec![(3, 3)]));
        assert_eq!(parse_ranges("1,3-4"), Some(vec![(1, 1), (3, 4)]));
    }

    #[test]
    fn rejects_broken_comma_separated_numbers() {
        assert_eq!(parse_ranges("1,"), None);
        assert_eq!(parse_ranges(",1"), None);
        assert_eq!(parse_ranges("1,,3"), None);
        assert_eq!(parse_ranges("1,a"), None);
        assert_eq!(parse_ranges("1,4-2"), None);
    }

    #[test]
    fn keeps_out_of_range_numbers() {
        let files = strings(&["a.rs"]);
        let args = strings(&["0", "2", "1-5", "1,2"]);
        assert_eq!(expand(&args, &files), args);
    }
}
