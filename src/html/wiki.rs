use std::collections::HashMap;

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
	fn parse(
		data: String,
		lib: Option<&Library>,
		path: Utf8PathBuf,
		hash: HashMap<Utf8PathBuf, Utf8PathBuf>,
	) -> (Outline, String, Option<Vec<String>>) {
		crate::text::md::parse(data, lib, path, hash)
	}

	fn render<'s, 'p, 'html>(
		self,
		sack: &'s Sack,
		parsed: impl Renderable + 'p,
		outline: Outline,
		bib: Option<Vec<String>>,
	) -> impl Renderable + 'html
	where
		's: 'html,
		'p: 'html,
	{
		wiki(self, sack, parsed, outline, bib)
	}

	fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
		Some(Linkable::Link(Link {
			path,
			name: self.title.to_owned(),
			desc: None,
		}))
	}
}

fn wiki<'s, 'p, 'html>(
	matter: Wiki,
	sack: &'s Sack,
	parsed: impl Renderable + 'p,
	_: Outline,
	bib: Option<Vec<String>>,
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	let heading = matter.title.clone();
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
					h1 #top { (heading) }
				}
				section .wiki-article__markdown.markdown {
					(parsed)
				}

				@if let Some(bib) = bib {
					(crate::html::misc::show_bibliography(bib))
				}
			}
		}
	);

	crate::html::page(sack, main, matter.title.to_owned())
}
