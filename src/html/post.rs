use std::collections::HashMap;

use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};
use serde::Deserialize;

use crate::pipeline::{Content, Sack};
use crate::text::md::Outline;
use crate::{LinkDate, Linkable};

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Post {
	pub(crate) title: String,
	#[serde(with = "super::isodate")]
	pub(crate) date: DateTime<Utc>,
	pub(crate) desc: Option<String>,
}

impl Content for Post {
	fn parse(
		data: String,
		lib: Option<&Library>,
		dir: Utf8PathBuf,
		hash: HashMap<Utf8PathBuf, Utf8PathBuf>,
	) -> (Outline, String, Option<Vec<String>>) {
		crate::text::md::parse(data, lib, dir, hash)
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
		post(self, sack, parsed, outline, bib)
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

pub fn post<'s, 'p, 'html>(
	fm: Post,
	sack: &'s Sack,
	content: impl Renderable + 'p,
	outline: Outline,
	bib: Option<Vec<String>>,
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	let heading = fm.title.clone();
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
					h1 #top { (heading) }
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

	crate::html::page(sack, main, fm.title.clone())
}
