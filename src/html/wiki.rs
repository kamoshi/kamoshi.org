use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::{
    gen::{Sack, TreePage},
    html::page,
    md::Wiki,
    text::md::Outline
};



fn tree(sack: &Sack) -> impl Renderable {
    let tree = sack.get_tree("wiki/**/*.html");

    maud_move!(
        h2 .link-tree__heading {
          // {pages.chain(x => x.prefix)
          //   .map(pathify)
          //   .mapOrDefault(href =>
          //     <a class="link-tree__heading-text" href={href}>{heading}</a>,
          //     <span class="link-tree__heading-text">{heading}</span>
          // )}
        }
        nav .link-tree__nav {
            (list(&tree))
        }
    )
}

fn list(tree: &TreePage) -> impl Renderable + '_ {
    let subs = {
        let mut subs: Vec<_> = tree.subs.iter().collect();
        subs.sort_by(|a, b| a.0.cmp(b.0));
        subs
    };

    maud_move!(
        ul .link-tree__nav-list {
            @for (key, next) in subs {
                li .link-tree__nav-list-item {
                    span .link-tree__nav-list-text {
                        @if let Some(ref link) = next.link {
                            a .link-tree__nav-list-text.link href=(link.path.as_str()) {
                                (&link.name)
                            }
                        } @else {
                            span .link-tree__nav-list-text {
                                (key)
                            }
                        }
                    }
                    @if !next.subs.is_empty() {
                        (list(next))
                    }
                }
            }
        }
    )
}


pub fn wiki<'data, 'html, 'sack, T>(
    fm: &'data Wiki,
    content: T,
    _: Outline,
    sack: &'sack Sack,
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
                        (tree(sack))
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
            }
        }
    );

    page(&fm.title, main)
}
