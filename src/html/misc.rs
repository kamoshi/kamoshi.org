use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::Outline;
use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};

use crate::{model::Wiki, Link, MySack};

/// Render the outline for a document
pub(crate) fn show_outline(outline: Outline) -> impl Renderable {
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

pub(crate) fn emit_bibliography(bib: Vec<String>) -> impl Renderable {
	maud_move!(
		section .bibliography.markdown {
			h2 {
				"Bibliography"
			}
			ol {
				@for item in bib {
					li {
						(Raw(item))
					}
				}
			}
		}
	)
}

#[derive(Debug)]
pub struct TreePage {
	pub link: Option<Link>,
	pub subs: HashMap<String, TreePage>,
}

impl TreePage {
	fn new() -> Self {
		TreePage {
			link: None,
			subs: HashMap::new(),
		}
	}

	fn add_link(&mut self, link: &Link) {
		let mut ptr = self;
		for part in link.path.iter().skip(1) {
			ptr = ptr.subs.entry(part.to_string()).or_insert(TreePage::new());
		}
		ptr.link = Some(link.clone());
	}

	fn from_iter(iter: impl Iterator<Item = Link>) -> Self {
		let mut tree = Self::new();
		for link in iter {
			tree.add_link(&link);
		}

		tree
	}
}

/// Render the page tree
pub(crate) fn show_page_tree<'a>(sack: &'a MySack, glob: &'a str) -> impl Renderable + 'a {
	let tree =
		TreePage::from_iter(
			sack.get_meta::<Wiki>(glob)
				.into_iter()
				.map(|(path, meta)| Link {
					path: Utf8Path::new("/").join(path.parent().unwrap()),
					name: meta.title.clone(),
					desc: None,
				}),
		);

	let parts = {
		let mut parts = sack.path.iter().skip(1).collect::<Vec<_>>();
		parts.insert(0, "wiki");
		parts
	};

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
			(show_page_tree_level(&tree, &parts))
		}
	)
}

fn show_page_tree_level<'a, 'b, 'c>(tree: &'a TreePage, parts: &'a [&str]) -> impl Renderable + 'b
where
	'a: 'b,
{
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
					@if key == parts[0] && !next.subs.is_empty()  {
						(show_page_tree_level(next, &parts[1..]))
					}
				}
			}
		}
	)
}
