use anyhow::Result;

use crate::cache;

/// "3" → Some((3, 3))、"2-4" → Some((2, 4))、それ以外 → None
fn parse_range(arg: &str) -> Option<(usize, usize)> {
    todo!()
}

/// 番号の引数をファイルパスに置き換える
/// 範囲外の番号や番号でない引数は、そのまま残す
pub fn expand(args: &[String], files: &[String]) -> Vec<String> {
    todo!()
}

/// 番号の引数があればキャッシュを読み込んで展開する
pub fn expand_with_cache(args: &[String]) -> Result<Vec<String>> {
    let has_number = args.iter().any(|arg| parse_range(arg).is_some());
    if !has_number {
        return Ok(args.to_vec());
    }

    let files = cache::load()?;
    Ok(expand(args, &files))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用に &str の配列を Vec<String> に変換する
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
    fn keeps_out_of_range_numbers() {
        let files = strings(&["a.rs"]);
        let args = strings(&["0", "2", "1-5"]);
        assert_eq!(expand(&args, &files), args);
    }
}
