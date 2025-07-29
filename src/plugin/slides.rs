use std::fmt::Write as _;

use camino::Utf8Path;
use hauchiwa::loader::{self, Content, Script, yaml};
use hauchiwa::{Page, Plugin, RuntimeError};
use hayagriva::Library;
use hypertext::{Raw, prelude::*};

use crate::model::Slideshow;
use crate::{CONTENT, Context, Global, LinkDate};

use super::{make_bare, to_list};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            loader::glob_content(CONTENT, "slides/**/*.md", yaml::<Slideshow>),
            loader::glob_content(CONTENT, "slides/**/*.lhs", yaml::<Slideshow>),
        ])
        .add_task("slides", |ctx| {
            let mut pages = vec![];

            // render individual pages
            for item in ctx.glob_with_file::<Content<Slideshow>>("slides/**/*")? {
                let mark = parse(&ctx, &item.data.text, &item.file.area, None)?;
                let html = render(&ctx, &item.data.meta, &mark)?.render();

                pages.push(Page::html(&item.file.area, html))
            }

            // render list
            {
                let data = ctx
                    .glob_with_file::<Content<Slideshow>>("slides/**/*")?
                    .into_iter()
                    .map(LinkDate::from)
                    .collect();

                let html = to_list(&ctx, data, "Slideshows".into(), "/slides/rss.xml")?.render();

                pages.push(Page::html("slides", html));
            }

            // render feed
            {
                let feed = crate::rss::generate_feed::<Content<Slideshow>>(
                    ctx,
                    "slides",
                    "Kamoshi.org Slides",
                )?;

                pages.push(feed);
            }

            Ok(pages)
        });
});

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles.scss", "reveal/reveal.scss"];

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
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let script = ctx.get::<Script>("slides/main.ts")?;
    let script = vec![script.path.to_string()];

    let html = maud!(
        div .reveal {
            div .slides {
                (Raw(slides))
            }
        }
    );

    make_bare(ctx, html, fm.title.clone(), STYLES, script.into())
}
