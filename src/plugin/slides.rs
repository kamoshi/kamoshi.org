use std::fmt::Write as _;

use camino::Utf8PathBuf;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::generic::DocumentMeta;
use hauchiwa::loader::{Image, Script, Stylesheet, TemplateEnv};
use hauchiwa::{Tracker, prelude::*};
use minijinja::Value;

use crate::model::Slideshow;
use crate::plugin::to_list;
use crate::props::PropsSlideshow;
use crate::{Context, Global, Link, LinkDate};

pub fn add_slides(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
    scripts: Many<Script>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let md = config
        .load_documents::<Slideshow>()
        .glob("content/slides/**/*.md")?
        .glob("content/slides/**/*.lhs")?
        .offset("content")
        .register();

    let handle = config
        .task()
        .using((templates, md, images, styles, scripts))
        .merge(|ctx, (templates, md, images, styles, scripts)| {
            let mut pages = vec![];

            let documents = md.values().collect::<Vec<_>>();

            {
                let styles = &[
                    styles.get("styles/styles.scss")?,
                    styles.get("styles/reveal/reveal.scss")?,
                ];

                let scripts = &[scripts.get("scripts/slides/main.ts")?];

                for document in &documents {
                    let text = parse(&document.text, &document.meta, None, Some(&images))?;
                    let html = render(ctx, templates, &document.matter, &text, styles, scripts)?;

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
                            path: Utf8PathBuf::from(&item.meta.href),
                            name: item.matter.title.clone(),
                            desc: item.matter.desc.clone(),
                        },
                        date: item.matter.date.to_utc(),
                    })
                    .collect();

                let html = to_list(
                    ctx,
                    templates,
                    data,
                    "Slideshows".into(),
                    "/slides/rss.xml",
                    styles,
                )?;

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
        });

    Ok(handle)
}

pub fn parse(
    text: &str,
    meta: &DocumentMeta,
    library: Option<&hayagriva::Library>,
    images: Option<&Tracker<Image>>,
) -> Result<String, RuntimeError> {
    let mut buff = String::new();

    for stack in text.split("\n-----\n") {
        buff.push_str("<section>");

        for slide in stack.split("\n---\n") {
            let article = crate::md::parse(slide, meta, None, images, library)?;
            write!(buff, "<section>{}</section>", article.html)?;
        }

        buff.push_str("</section>");
    }

    Ok(buff)
}

pub fn render(
    ctx: &Context,
    templates: &TemplateEnv,
    fm: &Slideshow,
    slides: &str,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<String, RuntimeError> {
    let props = PropsSlideshow {
        head: super::make_props_head(ctx, fm.title.clone(), styles, scripts)?,
        slides: Value::from_safe_string(slides.to_string()),
    };

    let tmpl = templates.get_template("slideshow.jinja")?;
    Ok(tmpl.render(&props)?)
}
