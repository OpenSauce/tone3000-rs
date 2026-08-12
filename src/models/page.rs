//! Paginated response envelope used by list/search endpoints.

use serde::Deserialize;

/// A page of results plus pagination metadata, matching the API's
/// `{ data, page, page_size, total, total_pages }` envelope.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// The results on this page.
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
    /// Which page this is, counting from 1.
    #[serde(default)]
    pub page: u32,
    /// How many results a full page holds.
    #[serde(default)]
    pub page_size: u32,
    /// Matching results across all pages.
    #[serde(default)]
    pub total: u64,
    /// How many pages the results span.
    ///
    /// Defaults to 0 if the API omits it, which makes [`has_next`](Self::has_next) report
    /// `false` — so a `while page.has_next()` loop stops rather than spinning.
    #[serde(default)]
    pub total_pages: u32,
}

impl<T> Page<T> {
    /// True if a further page follows this one.
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// True if a page precedes this one.
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_envelope() {
        let p: Page<u64> = serde_json::from_str(
            r#"{"data":[1,2],"page":1,"page_size":2,"total":5,"total_pages":3}"#,
        )
        .unwrap();
        assert_eq!(p.data, vec![1, 2]);
        assert_eq!(p.page, 1);
        assert_eq!(p.page_size, 2);
        assert_eq!(p.total, 5);
        assert_eq!(p.total_pages, 3);
    }

    #[test]
    fn has_next_and_prev_track_position() {
        let mid: Page<u64> =
            serde_json::from_str(r#"{"data":[],"page":2,"total_pages":3}"#).unwrap();
        assert!(mid.has_next());
        assert!(mid.has_prev());

        let first: Page<u64> =
            serde_json::from_str(r#"{"data":[],"page":1,"total_pages":3}"#).unwrap();
        assert!(first.has_next());
        assert!(!first.has_prev());

        let last: Page<u64> =
            serde_json::from_str(r#"{"data":[],"page":3,"total_pages":3}"#).unwrap();
        assert!(!last.has_next());
        assert!(last.has_prev());
    }

    #[test]
    fn empty_page_has_no_neighbours() {
        // An empty result set reports page 0 of 0; neither direction should be offered.
        let empty: Page<u64> = serde_json::from_str(r#"{"data":[]}"#).unwrap();
        assert!(!empty.has_next());
        assert!(!empty.has_prev());
    }

    #[test]
    fn tolerates_missing_fields() {
        let p: Page<u64> = serde_json::from_str(r#"{"data":[]}"#).unwrap();
        assert!(p.data.is_empty());
        assert_eq!(p.total_pages, 0);
    }
}
