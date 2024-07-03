use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, Renderable, GlobalAttributes, Raw};
use serde::Deserialize;

use crate::pipeline::{Content, Sack};
use crate::text::md::Outline;
use crate::{Link, LinkDate, Linkable};


/// Represents a slideshow
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Slideshow {
    pub title: String,
    #[serde(with = "super::isodate")]
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
}

impl Content for Slideshow {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        _: Outline,
        _: &'s Sack,
        _bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        show(self, content)
    }

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
        Some(Linkable::Date(LinkDate {
            link: Link {
                path,
                name: self.title.to_owned(),
                desc: self.desc.to_owned(),
            },
            date: self.date.to_owned(),
        }))
    }

    fn parse(data: &str, _: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
        let html = data
            .split("\n-----\n")
            .map(|chunk| chunk.split("\n---\n").map(|s| crate::text::md::parse(s, None)).map(|e| e.1).collect::<Vec<_>>())
            .map(|stack| match stack.len() > 1 {
                true  => format!("<section>{}</section>", stack.into_iter().map(|slide| format!("<section>{slide}</section>")).collect::<String>()),
                false => format!("<section>{}</section>", stack[0])
            })
            .collect::<String>();
        (Outline(vec![]), html, None)
    }
}

pub fn show<'data, 'show>(
    fm: &'data Slideshow,
    slides: impl Renderable + 'data
) -> impl Renderable + 'show
    where
        'data: 'show
{
    crate::html::bare(&fm.title, maud_move!(
        div .reveal {
            div .slides {
                (slides)
            }
        }

        script type="module" {
            (Raw("import 'reveal';"))
        }

        style {r#"
            .slides img {
              margin-left: auto;
              margin-right: auto;
              max-height: 60vh;
            }
        "#}
    ))
}
