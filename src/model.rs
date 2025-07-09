use std::str::FromStr;

use camino::Utf8Path;
use chrono::{DateTime, Utc};
use hauchiwa::{WithFile, plugin::content::Content};
use serde::Deserialize;

use crate::{Link, LinkDate};

#[derive(Deserialize)]
pub struct Pubkey {
    pub fingerprint: String,
    pub data: String,
}

#[derive(Deserialize, Clone)]
pub struct MicroblogEntry {
    #[serde(with = "isodate")]
    pub date: DateTime<Utc>,
    pub text: String,
}

#[derive(Deserialize)]
pub struct Microblog {
    pub entries: Vec<MicroblogEntry>,
    pub data: String,
}

impl FromStr for MicroblogEntry {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, char::is_whitespace);
        let datetime_str = parts.next().unwrap_or("");
        let text = parts.next().unwrap_or("").trim_start();

        let date = DateTime::parse_from_rfc3339(datetime_str)?.with_timezone(&Utc);

        Ok(MicroblogEntry {
            date,
            text: text.to_string(),
        })
    }
}

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Home {}

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub struct Post {
    /// The title of the post shown as `<h1>` heading.
    pub title: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(with = "isodate")]
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
    pub tags: Option<Vec<String>>,
    pub scripts: Option<Vec<String>>,
}

impl From<WithFile<'_, Content<Post>>> for LinkDate {
    fn from(item: WithFile<Content<Post>>) -> Self {
        Self {
            link: Link {
                path: Utf8Path::new("/").join(&item.file.slug),
                name: item.data.meta.title.clone(),
                desc: item.data.meta.desc.clone(),
            },
            date: item.data.meta.date,
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

impl From<WithFile<'_, Content<Slideshow>>> for LinkDate {
    fn from(item: WithFile<Content<Slideshow>>) -> Self {
        Self {
            link: Link {
                path: Utf8Path::new("/").join(&item.file.slug),
                name: item.data.meta.title.clone(),
                desc: item.data.meta.desc.clone(),
            },
            date: item.data.meta.date,
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
