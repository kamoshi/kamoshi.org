use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::gen::Sack;
use crate::html::misc::{show_bibliography, show_outline};
use crate::html::page;
use crate::md::Post;
use crate::text::md::Outline;


pub fn post<'f, 'm, 's, 'html, T>(
    fm: &'f Post,
    content: T,
    outline: Outline,
    bib: Option<Vec<String>>,
    sack: &'s Sack,
) -> impl Renderable + 'html
    where
        'f: 'html,
        'm: 'html,
        's: 'html,
        T: Renderable + 'm
{
    let main = maud_move!(
        main .wiki-main {

            // Slide in/out for mobile
            input #wiki-aside-shown type="checkbox" hidden;

            aside .wiki-aside {
                // Slide button
                label .wiki-aside__slider for="wiki-aside-shown" {
                    img .wiki-icon src="/static/svg/double-arrow.svg" width="24" height="24";
                }
                (show_outline(outline))
            }

            article .wiki-article /*class:list={classlist)*/ {
                header class="markdown" {
                    h1 #top { (fm.title.clone()) }
                }
                section .wiki-article__markdown.markdown {
                    (content)
                }

                @if let Some(bib) = bib {
                    (show_bibliography(bib))
                }
            }
        }
    );

    page(&fm.title, main, sack.get_file())
}
