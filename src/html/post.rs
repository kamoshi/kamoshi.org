use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use serde::Deserialize;

use crate::pipeline::{Content, Sack};
use crate::text::md::Outline;
use crate::{Linkable, LinkDate};

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Post {
    pub(crate) title: String,
    #[serde(with = "super::isodate")]
    pub(crate) date: DateTime<Utc>,
    pub(crate) desc: Option<String>,
}

impl Content for Post {
    fn parse(data: &str, lib: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
        crate::text::md::parse(data, lib)
    }

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
        T: Renderable + 'm,
    {
        post(self, content, outline, bib, sack)
    }

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
        Some(Linkable::Date(LinkDate {
            link: crate::Link {
                path,
                name: self.title.to_owned(),
                desc: self.desc.to_owned(),
            },
            date: self.date.to_owned(),
        }))
    }
}

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
                (crate::html::misc::show_outline(outline))
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
