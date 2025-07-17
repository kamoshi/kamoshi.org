use std::{borrow::Cow, fmt::Write};

use camino::Utf8Path;
use hauchiwa::loader::Script;
use hayagriva::Library;
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud};
use sequoia_openpgp::anyhow;

use crate::{Bibliography, Context, Outline, model::Slideshow};

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/reveal/reveal.scss"];

pub fn parse_content(
    ctx: &Context,
    text: &str,
    path: &Utf8Path,
    library: Option<&Library>,
) -> anyhow::Result<(String, Outline, Bibliography)> {
    let mut buff = String::new();

    for stack in text.split("\n-----\n") {
        buff.push_str("<section>");
        for slide in stack.split("\n---\n") {
            let slide = crate::md::parse(ctx, slide, path, library)?.0;
            write!(buff, "<section>{slide}</section>").unwrap();
        }
        buff.push_str("</section>");
    }

    Ok((buff, Outline(vec![]), Bibliography(None)))
}

pub fn as_html(
    slides: &Slideshow,
    parsed: &str,
    ctx: &Context,
    _: Outline,
    _: Bibliography,
) -> String {
    show(ctx, slides, parsed)
}

pub fn show(ctx: &Context, fm: &Slideshow, slides: &str) -> String {
    let script = ctx.get::<Script>("scripts/src/slides/main.ts").unwrap();

    crate::html::bare(
        ctx,
        maud!(
            div .reveal {
                div .slides {
                    (Raw(slides))
                }
            }
        ),
        fm.title.clone(),
        STYLES,
        Cow::Borrowed(&[script.path.to_string()]),
    )
    .unwrap()
    .render()
    .into()
}
