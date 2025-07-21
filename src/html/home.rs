use hauchiwa::{
    TaskResult,
    loader::{Content, Svelte},
};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move, maud_static};
use sequoia_openpgp::anyhow;

use crate::{Context, LinkDate, md::parse, model::Post};

use super::page;

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles.scss", "layouts/home.scss", "components/kanji.scss"];

const INTRO: &str = r#"
## かもし

初めましての方は初めまして、ポーランド出身で日本語を勉強している人です。
間違いがあったらすみません。

こちらは個人的なウェブサイトで、「かもし」というのは個人サークル名といってもいいです。
日本語を練習するため日本語を使って色々なことを書きます。
英語も使います。趣味はプログラミングや日本語や日本の歌や同人など色々なことです。
質問があったらメールを送信してくれてください。
"#;

pub(crate) fn home(ctx: &Context, text: &str) -> TaskResult<String> {
    let intro = intro(ctx)?;
    let posts = latest_posts(ctx)?;
    let kanji = ctx.get::<Svelte<()>>("src/kanji/App.svelte")?;
    let kanji_html = (kanji.html)(&())?;

    let main = maud_move!(
        main .l-home {
            article .l-home__article.markdown {
                (Raw(text))
            }
            aside .l-home__aside {
                (intro)
                // (Raw(SECTION_IMAGE))
                section .p-card {
                    (Raw(&kanji_html))
                }
                (posts)
                (SECTION_BUTTONS)
            }
        }
    );

    let scripts = vec![kanji.init.to_string()];
    let rendered = page(ctx, main, "Home".into(), STYLES, scripts.into())?
        .render()
        .into();

    Ok(rendered)
}

fn intro(ctx: &Context) -> anyhow::Result<impl Renderable> {
    let (parsed, _, _) = parse(ctx, INTRO, "/".into(), None)?;

    let html = maud_move!(
        section .p-card.intro-jp lang="ja-JP" {
            (Raw(&parsed))
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

fn latest_posts(sack: &Context) -> TaskResult<impl Renderable> {
    let list = {
        let mut list: Vec<_> = sack
            .glob_with_file::<Content<Post>>("**")?
            .iter()
            .map(LinkDate::from)
            .collect();
        list.sort_by(|a, b| b.date.cmp(&a.date));
        list
    };

    let html = maud_move!(
        section .p-card {
            h2 .p-card__heading {
                "Latest"
            }
            ol .p-card__latest {
                @for link in list.iter().take(5) {
                    li {
                        a href=(link.link.path.as_str()) {
                            (&link.link.name)
                        }
                    }
                }
            }
        }
    );

    Ok(html)
}

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
