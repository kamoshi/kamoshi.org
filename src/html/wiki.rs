use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::gen::Sack;
use crate::html::misc::show_page_tree;
use crate::html::{misc::show_bibliography, page};
use crate::md::Wiki;
use crate::text::md::Outline;


pub fn wiki<'data, 'html, 'sack, T>(
    fm: &'data Wiki,
    content: T,
    _: Outline,
    sack: &'sack Sack,
    bib: Option<Vec<String>>,
) -> impl Renderable + 'html
    where
        'sack: 'html,
        'data: 'html,
        T: Renderable + 'data
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
                // Navigation tree
                section .link-tree {
                    div {
                        (show_page_tree(sack, "wiki/**/*.html"))
                    }
                }
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
