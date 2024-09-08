use camino::{Utf8Path, Utf8PathBuf};
use hauchiwa::{Bibliography, Content, Link, Linkable, Outline, Sack};
use hayagriva::Library;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};
use serde::Deserialize;

/// Represents a wiki page
#[derive(Deserialize, Debug, Clone)]
pub struct Wiki {
	pub title: String,
}

impl Content for Wiki {
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
		wiki(self, parsed, sack, outline, bibliography)
	}

	fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
		Some(Linkable::Link(Link {
			path,
			name: self.title.to_owned(),
			desc: None,
		}))
	}
}

fn wiki(
	matter: &Wiki,
	parsed: &str,
	sack: &Sack,
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
						(crate::html::misc::show_page_tree(sack, "wiki/**/*.html"))
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
		.render()
		.into()
}
