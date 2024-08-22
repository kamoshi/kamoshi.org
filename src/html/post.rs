use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Utc};
use hauchiwa::{Bibliography, Content, Link, LinkDate, Linkable, Outline, Sack};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};
use serde::Deserialize;

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Post {
	pub(crate) title: String,
	#[serde(with = "super::isodate")]
	pub(crate) date: DateTime<Utc>,
	pub(crate) desc: Option<String>,
}

impl Content for Post {
	fn parse_content(
		content: &str,
		sack: &Sack,
		path: &Utf8Path,
		library: Option<&Library>,
	) -> (String, Outline, Bibliography) {
		crate::text::md::parse(content, sack, path, library)
	}

	fn as_html(
		&self,
		parsed: &str,
		sack: &Sack,
		outline: Outline,
		bibliography: Bibliography,
	) -> String {
		post(self, parsed, sack, outline, bibliography).render().into()
	}

	fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
		Some(Linkable::Date(LinkDate {
			link: Link {
				path,
				name: self.title.to_owned(),
				desc: self.desc.to_owned(),
			},
			date: self.date.to_owned(),
		}))
	}
}

pub fn post<'s, 'p, 'html>(
	metadata: &'p Post,
	parsed: &'p str,
	sack: &'s Sack,
	outline: Outline,
	bibliography: Bibliography,
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	let heading = metadata.title.clone();
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
					(Raw(parsed))
				}

				@if let Some(bib) = bibliography.0 {
					(crate::html::misc::show_bibliography(bib))
				}
			}
		}
	);

	crate::html::page(sack, main, metadata.title.clone())
}

