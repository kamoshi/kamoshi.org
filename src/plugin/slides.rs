use std::fmt::Write as _;

use camino::Utf8Path;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Image, Registry, Script, Stylesheet};
use hauchiwa::page::{Page, absolutize};
use hauchiwa::task::Handle;
use hauchiwa::{SiteConfig, task};
use hayagriva::Library;
use hypertext::{Raw, prelude::*};

use crate::model::Slideshow;
use crate::plugin::to_list;
use crate::{Context, Global, Link, LinkDate};

use super::make_bare;

pub fn build_slides(
    config: &mut SiteConfig<Global>,
    images: Handle<Registry<Image>>,
    styles: Handle<Registry<Stylesheet>>,
    scripts: Handle<Registry<Script>>,
) -> Result<Handle<Vec<Page>>, HauchiwaError> {
    let md = config.load_frontmatter::<Slideshow>("content/slides/**/*.md")?;
    let hs = config.load_frontmatter::<Slideshow>("content/slides/**/*.lhs")?;

    Ok(task!(config, |ctx, md, hs, images, styles, scripts| {
        let mut pages = vec![];

        let content = [md.values(), hs.values()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/reveal/reveal.scss")?,
            ];

            let scripts = &[scripts.get("scripts/slides/main.ts")?];

            // render individual pages
            for item in &content {
                let mark = parse(&item.content, &item.path, None, Some(images))?;
                let html = render(ctx, &item.metadata, &mark, styles, scripts)?.render();

                pages.push(Page::html(item.path.strip_prefix("content/")?, html))
            }
        }

        // render list
        {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/list.scss")?,
            ];

            let data = content
                .iter()
                .map(|item| LinkDate {
                    link: Link {
                        path: absolutize("content/", &item.path),
                        name: item.metadata.title.clone(),
                        desc: item.metadata.desc.clone(),
                    },
                    date: item.metadata.date.to_utc(),
                })
                .collect();

            let html = to_list(ctx, data, "Slideshows".into(), "/slides/rss.xml", styles)?.render();

            pages.push(Page::html("slides", html));
        }

        //             // render feed
        //             {
        //                 let feed = crate::rss::generate_feed::<Content<Slideshow>>(
        //                     ctx,
        //                     "slides",
        //                     "Kamoshi.org Slides",
        //                 )?;

        //                 pages.push(feed);
        //             }

        Ok(pages)
    }))
}

pub fn parse(
    text: &str,
    path: &Utf8Path,
    library: Option<&Library>,
    images: Option<&Registry<Image>>,
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
                (Raw(slides))
            }
        }
    );

    make_bare(ctx, html, fm.title.clone(), styles, scripts)
}
