use camino::Utf8Path;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Image, Script, Stylesheet, Svelte, TemplateEnv};
use hauchiwa::prelude::*;
use minijinja::Value;

use crate::Context;
use crate::{Global, model::Home};
use crate::props::{PropsFooter, PropsHead, PropsHome, PropsNavItem, PropsNavbar};

const INTRO: &str = include_str!("./intro.md");

const LOGOTYPE_SVG: &str = include_str!("../../assets/logotype.svg");

const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("綴", "Posts", "/posts/"),
    ("映", "Slides", "/slides/"),
    ("創", "Projects", "/projects/"),
    ("葉", "Garden", "/wiki/"),
    ("想", "Journal", "/thoughts/"),
    ("跡", "Map", "/map/"),
    ("己", "About", "/about/"),
    ("索", "Search", "/search/"),
];


pub fn add_home(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
    svelte: Many<Svelte>,
) -> Result<One<Output>, HauchiwaError> {
    let docs = config
        .load_documents::<Home>()
        .source("content/index.md")
        .offset("content")
        .register()?;

    let task = config.task().using((templates, docs, images, styles, svelte)).merge(
        |ctx, (templates, docs, images, styles, svelte)| {
            let document = docs.get("content/index.md")?;

            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/home.scss")?,
                styles.get("styles/components/kanji.scss")?,
            ];

            let kanji = svelte.get("scripts/kanji/App.svelte")?;
            let kanji_html = (kanji.prerender)(&())?;

            let scripts = &[&kanji.hydration];

            let parsed =
                crate::md::parse(&document.text, &document.meta, None, Some(&images), None)?;

            let html = render(ctx, templates, &parsed.html, &kanji_html, styles, scripts)?;

            Ok(Output::html("", html))
        },
    );

    Ok(task)
}

pub(crate) fn render(
    ctx: &Context,
    templates: &TemplateEnv,
    text: &str,
    kanji: &str,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<String, RuntimeError> {
    let intro_html = comrak::markdown_to_html(INTRO, &comrak::Options::default());

    let repo_link = Utf8Path::new(&ctx.env.data.link)
        .join("tree")
        .join(&ctx.env.data.hash);

    let props = PropsHome {
        head: PropsHead {
            title: "Home".to_string(),
            generator: ctx.env.generator,
            importmap: Value::from_safe_string(ctx.importmap.to_json()?),
            styles: styles.iter().map(|s| s.path.to_string()).collect(),
            scripts: scripts.iter().map(|s| s.path.to_string()).collect(),
            refresh_script: ctx.env.get_refresh_script().map(Value::from_safe_string),
        },
        navbar: PropsNavbar {
            logotype_svg: Value::from_safe_string(LOGOTYPE_SVG.to_string()),
            items: NAV_ITEMS
                .iter()
                .map(|&(stamp, name, url)| PropsNavItem { stamp, name, url })
                .collect(),
        },
        footer: PropsFooter {
            year: ctx.env.data.year,
            repo_link: repo_link.to_string(),
            hash_short: ctx.env.data.hash[0..7].to_string(),
            date: ctx.env.data.date.clone(),
        },
        article: Value::from_safe_string(text.to_string()),
        kanji: Value::from_safe_string(kanji.to_string()),
        intro: Value::from_safe_string(intro_html),
    };

    let tmpl = templates.get_template("home.jinja")?;
    Ok(tmpl.render(&props)?)
}

// const SECTION_IMAGE: Rendered<&str> = {
//     maud_static!(
//         section .p-card.home-card-image {
//             h2 .p-card__heading {
//                 "Image of the Month"
//             }
//             a .home-card-image__link href="/static/IMG_20231029_111650.jpg" {
//                 img .home-card-image__image
//                     src="/static/IMG_20231029_111650.jpg"
//                     alt="Autumn park with colorful trees and fallen leaves";
//             }
//         }
//     )
// };

// fn latest_posts(sack: &Context) -> Result<impl Renderable, RuntimeError> {
//     let list = {
//         let mut list: Vec<_> = sack
//             .glob_with_file::<Content<Post>>("**")?
//             .iter()
//             .map(LinkDate::from)
//             .collect();
//         list.sort_by(|a, b| b.date.cmp(&a.date));
//         list
//     };

//     let html = maud!(
//         section .p-card {
//             h2 .p-card__heading {
//                 "Latest"
//             }
//             ol .p-card__latest {
//                 @for link in list.iter().take(5) {
//                     li {
//                         a href=(link.link.path.as_str()) {
//                             (&link.link.name)
//                         }
//                     }
//                 }
//             }
//         }
//     );

//     Ok(html)
// }
