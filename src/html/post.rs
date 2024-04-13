use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use crate::md::Post;
use crate::html::page;


pub fn post<'fm, 'md, 'post, T>(fm: &'fm Post, content: T) -> impl Renderable + 'post
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
