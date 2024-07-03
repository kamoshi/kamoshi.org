use camino::Utf8PathBuf;
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use serde::Deserialize;

use crate::pipeline::{Content, Sack};
use crate::text::md::Outline;
use crate::{Link, Linkable};

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Wiki {
    pub title: String,
}

impl Content for Wiki {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        outline: Outline,
        sack: &'s Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        wiki(self, content, outline, sack, bib)
    }

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
        Some(Linkable::Link(Link {
            path,
            name: self.title.to_owned(),
            desc: None,
        }))
    }

    fn parse(data: &str, lib: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
        crate::text::md::parse(data, lib)
    }
}

fn wiki<'data, 'html, 'sack, T>(
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
                        (crate::html::misc::show_page_tree(sack, "wiki/**/*.html"))
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
                    (crate::html::misc::show_bibliography(bib))
                }
            }
        }
    );

    crate::html::page(&fm.title, main, sack.get_file())
}
