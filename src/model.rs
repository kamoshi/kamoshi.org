use camino::Utf8Path;
use chrono::{DateTime, Utc};
use hauchiwa::ViewPage;
use serde::Deserialize;

use crate::{Link, LinkDate};

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Home {}

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub struct Post {
    /// The title of the post shown as `<h1>` heading.
    pub title: String,
    #[serde(with = "isodate")]
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
    pub tags: Option<Vec<String>>,
    pub scripts: Option<Vec<String>>,
}

impl From<ViewPage<'_, Post>> for LinkDate {
    fn from(query: ViewPage<Post>) -> Self {
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

impl From<ViewPage<'_, Slideshow>> for LinkDate {
    fn from(query: ViewPage<Slideshow>) -> Self {
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

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub struct Project {
    pub title: String,
    /// List of technologies used
    pub tech: Vec<String>,
    pub link: String,
    pub desc: Option<String>,
}

// impl From<QueryContent<'_, Project>> for LinkDate {
//     fn from(query: QueryContent<Project>) -> Self {
//         Self {
//             link: Link {
//                 path: Utf8Path::new("/").join(query.slug),
//                 name: query.meta.title.clone(),
//                 desc: query.meta.desc.clone(),
//             },
//             date: query.meta.date,
//         }
//     }
// }

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
