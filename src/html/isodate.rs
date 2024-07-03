//! This module is supplementary to Serde, it allows you tu parse JS dates.

use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Deserializer};

// pub fn serialize<S>(
//     date: &DateTime<Utc>,
//     serializer: S,
// ) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let s = date.to_rfc3339();
//     serializer.serialize_str(&s)
// }

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = chrono::DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
    Ok(dt.into())
}
