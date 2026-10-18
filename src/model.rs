use std::str::FromStr;

use jiff::Timestamp;
// use hauchiwa::{WithFile, loader::Content};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Pubkey {
    pub fingerprint: String,
    pub data: String,
}

#[derive(Deserialize, Clone)]
pub struct MicroblogEntry {
    #[serde(with = "isodate")]
    pub date: Timestamp,
    pub text: String,
}

#[derive(Clone, Deserialize)]
pub struct Microblog {
    pub entries: Vec<MicroblogEntry>,
    pub data: String,
}

impl FromStr for MicroblogEntry {
    type Err = jiff::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, char::is_whitespace);
        let datetime_str = parts.next().unwrap_or("");
        let text = parts.next().unwrap_or("").trim_start();

        let date = datetime_str.parse::<Timestamp>()?;

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
    pub date: Timestamp,
    pub desc: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub scripts: Option<Vec<String>>,
}

// impl From<&WithFile<'_, Content<Post>>> for LinkDate {
//     fn from(item: &WithFile<Content<Post>>) -> Self {
//         Self {
//             link: Link {
//                 path: Utf8Path::new("/").join(&item.file.area),
//                 name: item.data.meta.title.clone(),
//                 desc: item.data.meta.desc.clone(),
//             },
//             date: item.data.meta.date,
//         }
//     }
// }

/// Represents a slideshow
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Slideshow {
    pub title: String,
    #[serde(with = "isodate")]
    pub date: Timestamp,
    pub desc: Option<String>,
}

// impl From<WithFile<'_, Content<Slideshow>>> for LinkDate {
//     fn from(item: WithFile<Content<Slideshow>>) -> Self {
//         Self {
//             link: Link {
//                 path: Utf8Path::new("/").join(&item.file.area),
//                 name: item.data.meta.title.clone(),
//                 desc: item.data.meta.desc.clone(),
//             },
//             date: item.data.meta.date,
//         }
//     }
// }

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub struct Project {
    pub title: String,
    /// List of technologies used
    pub tech: Vec<String>,
    /// External link; if absent, the markdown body is rendered as a page
    pub link: Option<String>,
    pub desc: Option<String>,
    pub scripts: Option<Vec<String>>,
    /// When true, the page footer is hidden
    pub hide_footer: Option<bool>,
}

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Wiki {
    pub title: String,
}

pub mod svelte {
    use serde::Serialize;
    use ts_rs::TS;

    #[derive(Serialize, TS)]
    #[ts(export, export_to = "../scripts/search/props.ts")]
    #[allow(dead_code)]
    pub struct Search {
        pub test: &'static str,
    }
}

mod isodate {
    use jiff::Timestamp;
    use serde::{self, Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Timestamp>().map_err(serde::de::Error::custom)
    }
}
