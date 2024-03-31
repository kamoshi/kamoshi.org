use hypertext::{html_elements, maud_move, Renderable, GlobalAttributes, Raw};

use crate::md::Slide;

use super::page;


pub fn show<'data, 'show>(
    fm: &'data Slide,
    slides: impl Renderable + 'data
) -> impl Renderable + 'show
    where
        'data: 'show
{
    page::bare(&fm.title, maud_move!(
        div .reveal {
            div .slides {
                (slides)
            }
        }

        script type="module" {
            (Raw("import 'reveal';"))
        }

        style {r#"
            .slides img {
              margin-left: auto;
              margin-right: auto;
              max-height: 60vh;
            }
        "#}
    ))
}
