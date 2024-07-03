use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;


pub fn preflight<T>(raw: &str) -> (T, String)
where
    T: for<'de> Deserialize<'de>,
{
    let matter = Matter::<YAML>::new();
    let result = matter.parse(raw);

    (
        // Just the front matter
        result.data.unwrap().deserialize::<T>().unwrap(),
        // The actual markdown content
        result.content,
    )
}

mod isodate {
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

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = chrono::DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
        Ok(dt.into())
    }
}
