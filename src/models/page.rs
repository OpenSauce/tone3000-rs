//! Paginated response envelope used by list/search endpoints.

use serde::Deserialize;

/// A page of results plus pagination metadata, matching the API's
/// `{ data, page, page_size, total, total_pages }` envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub page_size: u32,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub total_pages: u32,
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
    fn tolerates_missing_fields() {
        let p: Page<u64> = serde_json::from_str(r#"{"data":[]}"#).unwrap();
        assert!(p.data.is_empty());
        assert_eq!(p.total_pages, 0);
    }
}
