use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Image, Script, Stylesheet, Svelte};
use hauchiwa::page::Output;
use hauchiwa::task::Handle;
use hauchiwa::{Blueprint, task};
use hypertext::{Raw, maud_static, prelude::*};

use crate::Context;
use crate::markdown::parse;
use crate::{Global, model::Home};

use super::make_page;

pub fn build_home(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
    svelte: Handle<Assets<Svelte>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let docs = config.load_documents::<Home>("content/index.md")?;

    Ok(task!(config, |ctx, docs, images, styles, svelte| {
        let document = docs.get("content/index.md")?;

        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/home.scss")?,
            styles.get("styles/components/kanji.scss")?,
        ];

        let kanji = svelte.get("scripts/kanji/App.svelte")?;
        let kanji_html = (kanji.html)(&())?;

        let scripts = &[&kanji.init];

        let article = parse(&document.body, &document.path, None, Some(images))?;
        let html = render(ctx, &article.text, &kanji_html, styles, scripts)?;

        Ok(vec![Output::html("", html)])
    }))
}

const INTRO: &str = r#"
## かもし

初めましての方は初めまして、ポーランド出身で日本語を勉強している人です。
間違いがあったらすみません。

こちらは個人的なウェブサイトで、「かもし」というのは個人サークル名といってもいいです。
日本語を練習するため日本語を使って色々なことを書きます。
英語も使います。趣味はプログラミングや日本語や日本の歌や同人など色々なことです。
質問があったらメールを送信してくれてください。
"#;

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
                (Raw(text))
            }
            aside .l-home__aside {
                (intro)
                // (Raw(SECTION_IMAGE))
                section .p-card {
                    (Raw(kanji))
                }
                // (posts)
                (SECTION_BUTTONS)
            }
        }
    );

    let rendered = make_page(ctx, main, "Home".into(), styles, scripts)?
        .render()
        .into();

    Ok(rendered)
}

fn intro(_: &Context) -> Result<impl Renderable, RuntimeError> {
    let article = parse(INTRO, "".into(), None, None)?;

    let html = maud!(
        section .p-card.intro-jp lang="ja-JP" {
            (Raw(&article.text))
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
