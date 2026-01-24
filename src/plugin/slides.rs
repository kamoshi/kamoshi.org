use std::fmt::Write as _;

use camino::{Utf8Path, Utf8PathBuf};
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Image, Script, Stylesheet};
use hauchiwa::{Blueprint, Handle, Output, task};
use hayagriva::Library;
use hypertext::{Raw, prelude::*};

use crate::model::Slideshow;
use crate::plugin::to_list;
use crate::{Context, Global, Link, LinkDate};

use super::make_bare;

pub fn build_slides(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
    scripts: Handle<Assets<Script>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let md = config.load_documents::<Slideshow>("content/slides/**/*.md")?;
    let hs = config.load_documents::<Slideshow>("content/slides/**/*.lhs")?;

    Ok(task!(config, |ctx, md, hs, images, styles, scripts| {
        let mut pages = vec![];

        let documents = [md.values(), hs.values()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/reveal/reveal.scss")?,
            ];

            let scripts = &[scripts.get("scripts/slides/main.ts")?];

            for document in &documents {
                let text = parse(&document.body, &document.path, None, Some(images))?;
                let html = render(ctx, &document.metadata, &text, styles, scripts)?
                    .render()
                    .into_inner();

                pages.push(
                    document
                        .output()
                        .strip_prefix("content")?
                        .html()
                        .content(html),
                );
            }
        }

        // render list
        {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/list.scss")?,
            ];

            let data = documents
                .iter()
                .map(|item| LinkDate {
                    link: Link {
                        path: Utf8PathBuf::from(item.href("content")),
                        name: item.metadata.title.clone(),
                        desc: item.metadata.desc.clone(),
                    },
                    date: item.metadata.date.to_utc(),
                })
                .collect();

            let html = to_list(ctx, data, "Slideshows".into(), "/slides/rss.xml", styles)?
                .render()
                .into_inner();

            pages.push(Output::html("slides", html));
        }

        // render feed
        {
            pages.push(crate::rss::generate_feed(
                &documents,
                "slides",
                "Kamoshi.org Slides",
            ));
        }

        Ok(pages)
    }))
}

pub fn parse(
    text: &str,
    path: &Utf8Path,
    library: Option<&Library>,
    images: Option<&Assets<Image>>,
) -> Result<String, RuntimeError> {
    let mut buff = String::new();

    for stack in text.split("\n-----\n") {
        buff.push_str("<section>");
        for slide in stack.split("\n---\n") {
            let article = crate::markdown::parse(slide, path, library, images)?;
            write!(buff, "<section>{}</section>", article.text).unwrap();
        }
        buff.push_str("</section>");
    }

    Ok(buff)
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    fm: &'ctx Slideshow,
    slides: &'ctx str,
    styles: &'ctx [&Stylesheet],
    scripts: &'ctx [&Script],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let html = maud!(
        div .reveal {
            div .slides {
                (Raw::dangerously_create(slides))
            }
        }
    );

    make_bare(ctx, html, fm.title.clone(), styles, scripts)
}
