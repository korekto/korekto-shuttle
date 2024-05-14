use serde::{Deserialize, Deserializer, Serializer};
use sqlx::PgPool;
use std::fmt;

use crate::repository::Repository;

pub mod dtos;
mod find_user_by_id;
mod schedule_grading;
pub(crate) mod trackable;
mod user_assignments;
mod user_modules;

#[derive(Clone)]
pub struct Service {
    pub repo: Repository,
}

impl Service {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        let repo = Repository::new(pool);
        Self { repo }
    }
}

impl From<Repository> for Service {
    fn from(repo: Repository) -> Self {
        Self { repo }
    }
}

#[derive(PartialEq, Eq)]
pub struct ObfuscatedStr(pub String);

impl ObfuscatedStr {
    pub fn new<T: Into<String>>(value: T) -> Self {
        Self(value.into())
    }
}

impl fmt::Debug for ObfuscatedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.collect_str(&self)
    }
}
impl fmt::Display for ObfuscatedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() <= 3 {
            write!(f, "{}", &self.0)
        } else {
            write!(f, "{}*****", &self.0[..3])
        }
    }
}

impl From<String> for ObfuscatedStr {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ObfuscatedStr> for String {
    fn from(value: ObfuscatedStr) -> Self {
        value.0
    }
}

impl<'a> From<&'a ObfuscatedStr> for &'a str {
    fn from(value: &'a ObfuscatedStr) -> Self {
        value.0.as_str()
    }
}

impl<'de> Deserialize<'de> for ObfuscatedStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize_in_place(deserializer, &mut place.0)
    }
}

#[derive(Debug)]
pub enum SyncError {
    AssignmentNotFound,
    UserInstallationUnknown,
    BadInstallationId,
    Unknown(anyhow::Error),
}

#[cfg(test)]
mod tests {
    use crate::service::ObfuscatedStr;
    use pretty_assertions::assert_eq;

    #[test]
    fn display_obfuscated_str_for_empty_str() {
        let result = format!("{}", ObfuscatedStr("".to_string()));
        assert_eq!(result, "");
    }

    #[test]
    fn display_obfuscated_str_for_short_str() {
        let result = format!("{}", ObfuscatedStr("ma".to_string()));
        assert_eq!(result, "ma");
    }

    #[test]
    fn display_obfuscated_str_for_long_str() {
        let result = format!("{}", ObfuscatedStr("machin bidule".to_string()));
        assert_eq!(result, "mac*****");
    }

    #[test]
    fn debug_obfuscated_str_uses_display() {
        let result = format!("{:?}", ObfuscatedStr("machin bidule".to_string()));
        assert_eq!(result, "mac*****");
    }

    #[test]
    fn deserialize_obfuscated_str() {
        let o: ObfuscatedStr = serde_json::from_str("\"toto\"").unwrap();
        assert_eq!(o, ObfuscatedStr("toto".to_string()));
    }
}
