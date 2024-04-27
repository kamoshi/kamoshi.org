use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::html::misc::{show_bibliography, show_outline};
use crate::html::page;
use crate::md::Post;
use crate::text::md::Outline;


pub fn post<'fm, 'md, 'post, T>(
    fm: &'fm Post,
    content: T,
    outline: Outline,
    bib: Option<Vec<String>>,
) -> impl Renderable + 'post
    where
        'fm: 'post,
        'md: 'post,
        T: Renderable + 'md
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

    page(&fm.title, main)
}
