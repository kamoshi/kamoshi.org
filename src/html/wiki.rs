use camino::Utf8Path;
use hauchiwa::Bibliography;
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};

use crate::{model::Wiki, MySack, Outline};

pub fn parse_content(
	content: &str,
	sack: &MySack,
	path: &Utf8Path,
	library: Option<&Library>,
) -> (String, Outline, Bibliography) {
	crate::text::md::parse(content, sack, path, library)
}

pub fn as_html(
	meta: &Wiki,
	parsed: &str,
	sack: &MySack,
	slug: &Utf8Path,
	outline: Outline,
	bibliography: Bibliography,
) -> String {
	wiki(meta, parsed, sack, slug, outline, bibliography)
}

fn wiki(
	matter: &Wiki,
	parsed: &str,
	sack: &MySack,
	slug: &Utf8Path,
	_: Outline,
	bibliography: Bibliography,
) -> String {
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
						(crate::html::misc::show_page_tree(slug, sack))
					}
				}
			}

			article .wiki-article /*class:list={classlist)*/ {
				header class="markdown" {
					h1 #top { (heading) }
				}
				section .wiki-article__markdown.markdown {
					(Raw(parsed))
				}

				@if let Some(bib) = bibliography.0 {
					(crate::html::misc::emit_bibliography(bib))
				}
			}
		}
	);

	crate::html::page(sack, main, matter.title.to_owned(), None)
		.unwrap()
		.render()
		.into()
}
