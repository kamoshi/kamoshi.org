use hauchiwa::loader::{self, Content, Svelte, yaml};
use hauchiwa::{Page, Plugin, RuntimeError};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move, maud_static};

use crate::markdown::parse;
use crate::model::Post;
use crate::{CONTENT, Global, model::Home};
use crate::{Context, LinkDate};

use super::make_page;

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "index.md", yaml::<Home>),
        ])
        .add_task("home", |ctx| {
            let item = ctx.glob_one_with_file::<Content<Home>>("")?;
            let article = parse(&ctx, &item.data.text, &item.file.area, None)?;
            let html = render(&ctx, &article.text)?;
            Ok(vec![Page::text("index.html", html)])
        });
});

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

pub(crate) fn render(ctx: &Context, text: &str) -> Result<String, RuntimeError> {
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
    let rendered = make_page(ctx, main, "Home".into(), STYLES, scripts.into())?
        .render()
        .into();

    Ok(rendered)
}

fn intro(ctx: &Context) -> Result<impl Renderable, RuntimeError> {
    let article = parse(ctx, INTRO, "/".into(), None)?;

    let html = maud_move!(
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

fn latest_posts(sack: &Context) -> Result<impl Renderable, RuntimeError> {
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
