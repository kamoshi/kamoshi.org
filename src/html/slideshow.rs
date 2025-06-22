use std::fmt::Write;

use camino::Utf8Path;
use hayagriva::Library;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud};

use crate::{Bibliography, Context, Outline, model::Slideshow};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/reveal/reveal.scss"];

const CSS: &str = r#"
.slides img {
	margin-left: auto;
	margin-right: auto;
	max-height: 60vh;
}
"#;

pub fn parse_content(
    content: &str,
    sack: &Context,
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
    sack: &Context,
    _: Outline,
    _: Bibliography,
) -> String {
    show(slides, sack, parsed)
}

pub fn show(fm: &Slideshow, sack: &Context, slides: &str) -> String {
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
        STYLES,
        Some(&["js/vanilla/reveal.js".into()]),
    )
    .unwrap()
    .render()
    .into()
}
