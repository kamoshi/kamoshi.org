use std::fmt::Write as _;

use camino::Utf8Path;
use hauchiwa::SiteConfig;
use hauchiwa::error::RuntimeError;
use hauchiwa::loader::{self, CSS, Content, JS, Registry, Svelte, glob_content};
use hauchiwa::page::Page;
use hauchiwa::task::Handle;
use hayagriva::Library;
use hypertext::{Raw, prelude::*};

use crate::model::Slideshow;
use crate::plugin::to_list;
use crate::{Context, Global, Link, LinkDate};

use super::make_bare;

pub fn build_slides(
    config: &mut SiteConfig<Global>,
    styles: Handle<Registry<CSS>>,
    scripts: Handle<Registry<JS>>,
) -> Handle<Vec<Page>> {
    let slides_md = glob_content::<_, Slideshow>(config, "content/slides/**/*.md");
    let slides_hs = glob_content::<_, Slideshow>(config, "content/slides/**/*.lhs");

    config.add_task(
        (slides_md, slides_hs, styles, scripts),
        |ctx, (slides_md, slides_hs, styles, scripts)| {
            let mut pages = vec![];

            let content = [slides_md.values(), slides_hs.values()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            {
                let styles = &[
                    styles.get("styles/styles.scss").unwrap(),
                    styles.get("styles/reveal/reveal.scss").unwrap(),
                ];

                let scripts = &[scripts.get("scripts/slides/main.ts").unwrap()];

                // render individual pages
                for item in &content {
                    let mark = parse(&ctx, &item.content, "".into(), None).unwrap();
                    let html = render(&ctx, &item.metadata, &mark, styles, scripts)
                        .unwrap()
                        .render();

                    pages.push(Page {
                        url: item.path.to_string(),
                        content: html.into_inner(),
                    })
                }
            }

            // render list
            {
                let data = content
                    .iter()
                    .map(|item| LinkDate {
                        link: Link {
                            path: item.path.clone(),
                            name: item.metadata.title.clone(),
                            desc: item.metadata.desc.clone(),
                        },
                        date: item.metadata.date.to_utc(),
                    })
                    .collect();

                let styles = &[
                    styles.get("styles/styles.scss").unwrap(),
                    styles.get("styles/layouts/list.scss").unwrap(),
                ];

                let html = to_list(&ctx, data, "Slideshows".into(), "/slides/rss.xml", styles)
                    .unwrap()
                    .render();

                pages.push(Page {
                    url: "slides".into(),
                    content: html.into_inner(),
                });
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

            pages
        },
    )
}

pub fn parse(
    ctx: &Context,
    text: &str,
    path: &Utf8Path,
    library: Option<&Library>,
) -> Result<String, RuntimeError> {
    let mut buff = String::new();

    for stack in text.split("\n-----\n") {
        buff.push_str("<section>");
        for slide in stack.split("\n---\n") {
            let article = crate::markdown::parse(ctx, slide, path, library)?;
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
    styles: &'ctx [&CSS],
    scripts: &'ctx [&JS],
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
