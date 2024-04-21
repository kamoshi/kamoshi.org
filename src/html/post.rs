use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use crate::md::Post;
use crate::html::page;
use crate::text::md::Outline;


pub fn tree(outline: Outline) -> impl Renderable {
    maud_move!(
        section .link-tree {
            h2 .link-tree__heading {
                a .link-tree__heading-text href="#top" { "Content" }
            }
            nav #table-of-contents .link-tree__nav {
                ul .link-tree__nav-list {
                    @for (title, id) in outline.0 {
                        li .link-tree__nav-list-item {
                            a .link-tree__nav-list-text.link href=(format!("#{id}")) {
                                (title)
                            }
                        }
                    }
                }
            }
        }
    )
}


pub fn post<'fm, 'md, 'post, T>(
    fm: &'fm Post,
    content: T,
    outline: Outline,
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
                (tree(outline))
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
