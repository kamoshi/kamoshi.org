use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::gen::{Sack, TreePage};
use crate::text::md::Outline;


/// Render the outline for a document
pub fn show_outline(outline: Outline) -> impl Renderable {
    maud_move!(
        section .link-tree {
            h2 .link-tree__heading {
                a .link-tree__heading-text href="#top" { "Content" }
            }
            nav #table-of-contents .link-tree__nav {
                ul .link-tree__nav-list {
                    @for (title, id) in outline.0 {
                        li .link-tree__nav-list-item {
                            a .link-tree__nav-list-text.link href=(format!("#{}", id)) {
                                (title)
                            }
                        }
                    }
                }
            }
        }
    )
}

/// Render the bibliography for a document
pub fn show_bibliography(bib: Vec<String>) -> impl Renderable {
    maud_move!(
        section .markdown {
            h2 {
                "Bibliography"
            }
            ol .bibliography {
                @for item in bib {
                    li {
                        (item)
                    }
                }
            }
        }
    )
}

/// Render the page tree
pub fn show_page_tree(sack: &Sack, glob: &str) -> impl Renderable {
    let tree = sack.get_tree(glob);

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
            (show_page_tree_level(&tree))
        }
    )
}

fn show_page_tree_level(tree: &TreePage) -> impl Renderable + '_ {
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
                        (show_page_tree_level(next))
                    }
                }
            }
        }
    )
}
