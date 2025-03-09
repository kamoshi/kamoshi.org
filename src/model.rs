use camino::Utf8Path;
use chrono::{DateTime, Utc};
use hauchiwa::QueryContent;
use serde::Deserialize;

use crate::{Link, LinkDate};

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Home {
    pub title: String,
}

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub struct Post {
    pub title: String,
    #[serde(with = "isodate")]
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
    pub tags: Option<Vec<String>>,
    pub scripts: Option<Vec<String>>,
}

impl From<QueryContent<'_, Post>> for LinkDate {
    fn from(query: QueryContent<Post>) -> Self {
        Self {
            link: Link {
                path: Utf8Path::new("/").join(query.slug),
                name: query.meta.title.clone(),
                desc: query.meta.desc.clone(),
            },
            date: query.meta.date,
        }
    }
}

/// Represents a slideshow
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Slideshow {
    pub title: String,
    #[serde(with = "isodate")]
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
}

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Wiki {
    pub title: String,
}

mod isodate {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = chrono::DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
        Ok(dt.into())
    }
}
