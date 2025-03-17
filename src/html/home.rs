use hauchiwa::{Sack, TaskResult};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud, maud_move};

use crate::{Global, LinkDate, md::parse, model::Post};

use super::page;

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles/styles.scss", "styles/layouts/home.scss"];

const INTRO: &str = r#"
## かもし

初めましての方は初めまして、ポーランド出身で日本語を勉強している人です。
間違いがあったらすみません。

こちらは個人的なウェブサイトで、「かもし」というのは個人サークル名といってもいいです。
日本語を練習するため日本語を使って色々なことを書きます。
英語も使います。趣味はプログラミングや日本語や日本の歌や同人など色々なことです。
質問があったらメールを送信してくれてください。
"#;

pub(crate) fn home(ctx: &Sack<Global>, text: &str) -> TaskResult<String> {
    let intro = intro(ctx);
    let image = monthly_image();
    let posts = latest_posts(ctx)?;

    let main = maud!(
        main .l-home {
            article .l-home__article.markdown {
                (Raw(text))
            }
            aside .l-home__aside {
                (intro)
                (image)
                (posts)
            }
        }
    );

    let rendered = page(ctx, main, "Home".into(), STYLES, None)?
        .render()
        .into();

    Ok(rendered)
}

fn intro(ctx: &Sack<Global>) -> impl Renderable {
    let (parsed, _, _) = parse(INTRO, ctx, "/".into(), None);

    maud!(
        section .p-card.intro-jp lang="ja-JP" {
            (Raw(parsed))
        }
    )
}

fn monthly_image() -> impl Renderable {
    maud!(
        section .p-card.home-card-image {
            h2 .p-card__heading {
                "Image of the Month"
            }
            a .home-card-image__link href="/static/IMG_20231029_111650.jpg" {
                img .home-card-image__image
                    src="/static/IMG_20231029_111650.jpg"
                    alt="Autumn park with colorful trees and fallen leaves";
            }
        }
    )
}

fn latest_posts(sack: &Sack<Global>) -> TaskResult<impl Renderable> {
    let list = {
        let mut list: Vec<_> = sack
            .query_content::<Post>("**")?
            .into_iter()
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
