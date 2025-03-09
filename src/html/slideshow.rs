use std::fmt::Write;

use camino::Utf8Path;
use hayagriva::Library;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud};

use crate::{Bibliography, MySack, Outline, model::Slideshow};

const CSS: &str = r#"
.slides img {
	margin-left: auto;
	margin-right: auto;
	max-height: 60vh;
}
"#;

pub fn parse_content(
    content: &str,
    sack: &MySack,
    path: &Utf8Path,
    library: Option<&Library>,
) -> (String, Outline, Bibliography) {
    let parsed = content
        .split("\n-----\n")
        .map(|chunk| {
            chunk
                .split("\n---\n")
                .map(|slide| crate::md::parse(slide, sack, path, library).0)
                .collect::<Vec<_>>()
        })
        .map(|stack| match stack.len() > 1 {
            true => {
                let mut buffer = String::from("<section>");

                for slide in stack {
                    write!(buffer, "<section>{}</section>", slide).unwrap();
                }

                buffer
            }
            false => format!("<section>{}</section>", stack[0]),
        })
        .collect::<String>();
    (parsed, Outline(vec![]), Bibliography(None))
}

pub fn as_html(
    slides: &Slideshow,
    parsed: &str,
    sack: &MySack,
    _: Outline,
    _: Bibliography,
) -> String {
    show(slides, sack, parsed)
}

pub fn show(fm: &Slideshow, sack: &MySack, slides: &str) -> String {
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
