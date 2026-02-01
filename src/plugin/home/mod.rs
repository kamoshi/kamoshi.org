use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Image, Script, Stylesheet, Svelte};
use hauchiwa::prelude::*;
use hypertext::{Raw, maud_static, prelude::*};

use crate::Context;
use crate::{Global, model::Home};

use super::make_page;

const INTRO: &str = include_str!("./intro.md");

const SECTION_BUTTONS: Raw<&str> = {
    maud_static!(
        section .p-card {
            h2 .p-card__heading {
                "Badges"
            }
            div .grid_88x31 {
                a href=r#"https://nixos.org"# title="Powered by NixOS" {
                    img .icon_88x31 src="/static/88x31/nixos.gif" width=88 height=31;
                }
                a href=r#"https://crates.io/crates/hauchiwa"# title="Built with Hauchiwa" {
                    img .icon_88x31 src="/static/88x31/hauchiwa.png" width=88 height=31;
                }
                a href=r#"https://www.mozilla.org/firefox/new"# title="Tested on Firefox" {
                    img .icon_88x31 src="/static/88x31/firefox.webp" width=88 height=31;
                }
                a href=r#"https://creativecommons.org/licenses/by/4.0/"# title="CC BY 4.0" {
                    img .icon_88x31 src="/static/88x31/cc-by.png" width=88 height=31;
                }
            }
        }
    )
};

pub fn add_home(
    config: &mut Blueprint<Global>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
    svelte: Many<Svelte>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let docs = config
        .load_documents::<Home>()
        .source("content/index.md")
        .offset("content")
        .register()?;

    let task = config
        .task()
        .depends_on((docs, images, styles, svelte))
        .run(|ctx, (docs, images, styles, svelte)| {
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

            let html = render(ctx, &parsed.html, &kanji_html, styles, scripts)?;

            Ok(vec![Output::html("", html)])
        });

    Ok(task)
}

pub(crate) fn render(
    ctx: &Context,
    text: &str,
    kanji: &str,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<String, RuntimeError> {
    let intro = intro(ctx)?;
    // let posts = latest_posts(ctx)?;

    let main = maud!(
        main .l-home {
            article .l-home__article.markdown {
                (Raw::dangerously_create(text))
            }
            aside .l-home__aside {
                (intro)
                // (Raw(SECTION_IMAGE))
                section .p-card {
                    (Raw::dangerously_create(kanji))
                }
                // (posts)
                (SECTION_BUTTONS)
            }
        }
    );

    let rendered = make_page(ctx, main, "Home".into(), styles, scripts)?
        .render()
        .into_inner();

    Ok(rendered)
}

fn intro(_: &Context) -> Result<impl Renderable, RuntimeError> {
    let article = comrak::markdown_to_html(INTRO, &comrak::Options::default());

    let html = maud!(
        section .p-card.intro-jp lang="ja-JP" {
            (Raw::dangerously_create(&article))
        }
    );

    Ok(html)
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
