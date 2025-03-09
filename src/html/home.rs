use camino::Utf8Path;
use hauchiwa::{Sack, TaskResult};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud, maud_move};

use crate::{Link, LinkDate, MyData, md::parse, model::Post};

const INTRO: &str = r#"
## かもし

初めましての方は初めまして、ポーランド出身で日本語を勉強している人です。
間違いがあったらすみません。

こちらは個人的なウェブサイトで、「かもし」というのは個人サークル名といってもいいです。
日本語を練習するため日本語を使って色々なことを書きます。
英語も使います。趣味はプログラミングや日本語や日本の歌や同人など色々なことです。
質問があったらメールを送信してくれてください。
"#;

fn intro(sack: &Sack<MyData>) -> impl Renderable {
    let (parsed, _, _) = parse(INTRO, sack, "".into(), None);
    maud!(
        section .p-card.intro-jp lang="ja-JP" {
            (Raw(parsed))
        }
    )
}

fn photo() -> impl Renderable {
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

fn latest(sack: &Sack<MyData>) -> impl Renderable {
    let links = {
        let mut list = sack
            .query_content::<Post>("**")
            .unwrap()
            .into_iter()
            .map(|query| LinkDate {
                link: Link {
                    path: Utf8Path::new("/").join(query.slug),
                    name: query.meta.title.clone(),
                    desc: query.meta.desc.clone(),
                },
                date: query.meta.date,
            })
            .collect::<Vec<_>>();
        list.sort_by(|a, b| b.date.cmp(&a.date));
        list
    };

    maud_move!(
        section .p-card {
            h2 .p-card__heading {
                "Latest"
            }
            ol .p-card__latest {
                @for link in links.iter().take(5) {
                    li {
                        a href=(link.link.path.as_str()) {
                            (&link.link.name)
                        }
                    }
                }
            }
        }
    )
}

pub(crate) fn home(sack: &Sack<MyData>, main: &str) -> TaskResult<String> {
    let main = maud!(
        main .l-home {
            article .l-home__article.markdown {
                (Raw(main))
            }
            aside .l-home__aside {
                (intro(sack))
                // (kanji())
                (photo())
                (latest(sack))
            }
        }
    );

    let rendered = crate::html::page(sack, main, "Home".into(), None)?
        .render()
        .into();

    Ok(rendered)
}
