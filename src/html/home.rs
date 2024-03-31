use hypertext::{html_elements, maud, maud_move, GlobalAttributes, Raw, Renderable};

use crate::md::render;

use super::page;


const INTRO: &str = r#"
## かもし

初めましての方は初めまして、ポーランド出身で日本語を勉強している人です。
間違いがあったらすみません。

こちらは個人的なウェブサイトで、「かもし」というのは個人サークル名といってもいいです。
日本語を練習するため日本語を使って色々なことを書きます。
英語も使います。趣味はプログラミングや日本語や日本の歌や同人など色々なことです。
質問があったらメールを送信してくれてください。
"#;


fn intro() -> impl Renderable {
    maud!(
        section .p-card.intro-jp lang="ja-JP" {
            (Raw(render(INTRO)))
        }
    )
}

fn kanji() -> impl Renderable {
    maud!(
        section .p-card {
            h2 .p-card__heading {
                "Kanji of the Day"
            }
            div {
                // <Widget client:load/>
            }
        }
    )
}

fn photo() -> impl Renderable {
    maud!(
        section .p-card.home-card-image {
            h2 .p-card__heading {
                "Image of the Month"
            }
            a .home-card-image__link href="TODO" {
                img .home-card-image__image
                    src="TODO"
                    alt="Autumn park with colorful trees and fallen leaves";
            }
        }
    )
}

pub fn home<'data, 'home, R>(main: R) -> impl Renderable + 'home
    where
        'data: 'home,
        R: Renderable + 'data
{
    let main = maud_move!(
        main .l-home {
            article .l-home__article.markdown {
                (main)
            }
            aside .l-home__aside {
                (intro())
                (kanji())
                (photo())
            }
        }
    );

    page("Home", main)
}