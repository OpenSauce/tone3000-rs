use serde::Deserialize;

use super::tone::{Metrics, Sort};

/// A TONE3000 user.
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub metrics: Metrics,
}

/// Parameters for [`crate::Client::users`].
#[derive(Debug, Clone, Default)]
pub struct UserListParams {
    pub sort: Option<Sort>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_tolerates_missing_optionals() {
        let u: User = serde_json::from_str(r#"{ "id": "u1" }"#).unwrap();
        assert_eq!(u.id, "u1");
        assert!(u.display_name.is_none());
    }
}
