use camino::Utf8Path;
use chrono::{DateTime, Utc};
use hauchiwa::{Bibliography, Outline, Sack};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};
use serde::Deserialize;

/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub struct Post {
	pub title: String,
	#[serde(with = "super::isodate")]
	pub date: DateTime<Utc>,
	pub desc: Option<String>,
	pub scripts: Option<Vec<String>>,
}

pub fn parse_content(
	content: &str,
	sack: &Sack,
	path: &Utf8Path,
	library: Option<&Library>,
) -> (String, Outline, Bibliography) {
	crate::text::md::parse(content, sack, path, library)
}

pub fn as_html(
	meta: &Post,
	parsed: &str,
	sack: &Sack,
	outline: Outline,
	bibliography: Bibliography,
) -> String {
	post(meta, parsed, sack, outline, bibliography)
		.unwrap()
		.render()
		.into()
}

pub fn post<'s, 'p, 'html>(
	meta: &'p Post,
	parsed: &'p str,
	sack: &'s Sack,
	outline: Outline,
	bibliography: Bibliography,
) -> Result<impl Renderable + 'html, String>
where
	's: 'html,
	'p: 'html,
{
	let main = maud_move!(
		main {
			(article(&meta.title, parsed, sack, outline, bibliography))
		}
	);

	crate::html::page(sack, main, meta.title.clone(), meta.scripts.as_deref())
}

pub fn article<'p, 's, 'html>(
	title: &'p str,
	parsed: &'p str,
	_: &'s Sack,
	outline: Outline,
	bibliography: Bibliography,
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	maud_move!(
		div .wiki-main {

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
					h1 #top { (title) }
				}
				section .wiki-article__markdown.markdown {
					(Raw(parsed))
				}

				@if let Some(bib) = bibliography.0 {
					(crate::html::misc::emit_bibliography(bib))
				}
			}
		}
	)
}
