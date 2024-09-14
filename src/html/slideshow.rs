use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Utc};
use hauchiwa::{Bibliography, Link, LinkDate, Linkable, Outline, Sack};
use hayagriva::Library;
use hypertext::{html_elements, maud, GlobalAttributes, Raw, Renderable};
use serde::Deserialize;

const CSS: &str = r#"
.slides img {
	margin-left: auto;
	margin-right: auto;
	max-height: 60vh;
}
"#;

/// Represents a slideshow
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Slideshow {
    pub title: String,
    #[serde(with = "super::isodate")]
    pub date: DateTime<Utc>,
    pub desc: Option<String>,
}

pub fn parse_content(
    content: &str,
    sack: &Sack,
    path: &Utf8Path,
    library: Option<&Library>,
) -> (String, Outline, Bibliography) {
    let parsed = content
        .split("\n-----\n")
        .map(|chunk| {
            chunk
                .split("\n---\n")
                .map(|slide| crate::text::md::parse(&slide, sack, path, library).0)
                .collect::<Vec<_>>()
        })
        .map(|stack| match stack.len() > 1 {
            true => format!(
                "<section>{}</section>",
                stack
                    .into_iter()
                    .map(|slide| format!("<section>{slide}</section>"))
                    .collect::<String>()
            ),
            false => format!("<section>{}</section>", stack[0]),
        })
        .collect::<String>();
    (parsed, Outline(vec![]), Bibliography(None))
}

pub fn as_html(
    slides: &Slideshow,
    parsed: &str,
    sack: &Sack,
    _: Outline,
    _: Bibliography,
) -> String {
    show(slides, sack, parsed)
}

pub fn as_link(slides: &Slideshow, path: Utf8PathBuf) -> Option<Linkable> {
    Some(Linkable::Date(LinkDate {
        link: Link {
            path,
            name: slides.title.to_owned(),
            desc: slides.desc.to_owned(),
        },
        date: slides.date.to_owned(),
    }))
}

pub fn show(fm: &Slideshow, sack: &Sack, slides: &str) -> String {
    crate::html::bare(
        sack,
        maud!(
            div .reveal {
                div .slides {
                    (Raw(slides))
                }
            }

            style { (Raw(CSS)) }
        ),
        fm.title.clone(),
        Some(&["reveal".into()]),
    )
    .unwrap()
    .render()
    .into()
}
