use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use crate::md::Wiki;
use crate::html::page;
use crate::text::md::Outline;


pub fn wiki<'data, 'html, T>(
    fm: &'data Wiki,
    content: T,
    outline: Outline,
) -> impl Renderable + 'html
    where
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
                // <Tree heading="Personal Wiki" pages={pages} headings={headings} />
            }

            article .wiki-article /*class:list={classlist)*/ {
                header class="markdown" {
                    h1 #top { (fm.title.clone()) }
                }
                section .wiki-article__markdown.markdown {
                    (content)
                }
            }
        }
    );

    page(&fm.title, main)
}
