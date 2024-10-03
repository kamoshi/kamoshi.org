use camino::Utf8Path;
use hauchiwa::{Bibliography, Outline};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};

use crate::{model::Post, MySack};

pub fn parse_content(
	content: &str,
	sack: &MySack,
	path: &Utf8Path,
	library: Option<&Library>,
) -> (String, Outline, Bibliography) {
	crate::text::md::parse(content, sack, path, library)
}

pub fn as_html(
	meta: &Post,
	parsed: &str,
	sack: &MySack,
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
	sack: &'s MySack,
	outline: Outline,
	bibliography: Bibliography,
) -> Result<impl Renderable + 'html, String>
where
	's: 'html,
	'p: 'html,
{
	let main = maud_move!(
		main {
			(article(meta, parsed, sack, outline, bibliography))
		}
	);

	crate::html::page(sack, main, meta.title.clone(), meta.scripts.as_deref())
}

pub fn article<'p, 's, 'html>(
	meta: &'p Post,
	parsed: &'p str,
	_: &'s MySack,
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

			(paper_page(meta, parsed, bibliography))
		}
	)
}

fn paper_page<'a>(meta: &'a Post, parsed: &'a str, bib: Bibliography) -> impl Renderable + 'a {
	maud_move!(
		article .wiki-article {
			header {
				h1 #top {
					(&meta.title)
				}
				div .line {
					div .date {
						(meta.date.format("%Y-%m-%d").to_string())
					}
					@if let Some(ref tags) = meta.tags {
						ul .tags {
							@for tag in tags {
								li { (tag) }
							}
						}
					}
				}
			}
			section .wiki-article__markdown.markdown {
				(Raw(parsed))
			}

			@if let Some(bib) = bib.0 {
				(crate::html::misc::emit_bibliography(bib))
			}
		}
	)
}
