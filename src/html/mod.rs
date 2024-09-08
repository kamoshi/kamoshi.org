mod home;
mod isodate;
mod list;
mod misc;
mod post;
mod slideshow;
mod special;
mod wiki;
mod head;

use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use hauchiwa::{Bibliography, Content, Link, LinkDate, Linkable, Outline, Sack};
use hayagriva::Library;
use hypertext::{html_elements, maud, maud_move, GlobalAttributes, Raw, Renderable};

pub(crate) use home::home;
use post::article;
pub(crate) use post::Post;
use serde::Deserialize;
pub(crate) use slideshow::Slideshow;
pub(crate) use wiki::Wiki;

fn navbar() -> impl Renderable {
	static ITEMS: &[(&str, &str)] = &[
		("Posts", "/posts/"),
		("Slides", "/slides/"),
		("Projects", "/projects/"),
		("Wiki", "/wiki/"),
		("Map", "/map/"),
		("About", "/about/"),
		("Search", "/search/"),
	];

	maud!(
		nav .p-nav {
			input #p-nav-toggle type="checkbox" hidden;

			div .p-nav__bar {
				a .p-nav__logo href="/" {
					img .p-nav__logo-icon height="48px" width="51px" src="/static/svg/aya.svg" alt="";
					div .p-nav__logo-text {
						div .p-nav__logo-main {
							(Raw(include_str!("logotype.svg")))
						}
						div #p-nav-splash .p-nav__logo-sub {
						  "夢現の遥か彼方"
						}
					}
				}

				label .p-nav__burger for="p-nav-toggle" tabindex="0" {
					span .p-nav__burger-icon {}
				}
			}

			menu .p-nav__menu {
				@for (name, url) in ITEMS {
					li .p-nav__menu-item {
						a .p-nav__menu-link href=(*url) {
							(*name)
						}
					}
				}
			}
		}
	)
}

pub fn footer<'s, 'html>(sack: &'s Sack) -> impl Renderable + 'html
where
	's: 'html,
{
	let copy = format!("Copyright &copy; {} Maciej Jur", &sack.ctx.year);
	let mail = "maciej@kamoshi.org";
	let href = format!("mailto:{}", mail);
	let link = Utf8Path::new(&sack.ctx.link)
		.join("src/commit")
		.join(&sack.ctx.hash);
	let link = match sack.get_file() {
		Some(path) => link.join(path),
		None => link,
	};

	maud_move!(
		footer .footer {
			div .left {
				div {
					(Raw(copy))
				}
				a href=(href)  {
					(mail)
				}
			}
			div .repo {
				a href=(link.as_str()) {
					(&sack.ctx.hash)
				}
				div {
					(&sack.ctx.date)
				}
			}
			a .right.footer__cc-wrap rel="license" href="http://creativecommons.org/licenses/by/4.0/" {
				img .footer__cc-stamp alt="Creative Commons License" width="88" height="31" src="/static/svg/by.svg";
			}
		}
	)
}

fn bare<'s, 'p, 'html>(
	sack: &'s Sack,
	main: impl Renderable + 'p,
	title: String,
	js: Option<&'s [String]>,
) -> Result<impl Renderable + 'html, String>
where
	's: 'html,
	'p: 'html,
{
    let head = head::render_head(sack, title, &[], js)?;

	Ok(maud_move!(
		(Raw("<!DOCTYPE html>"))
		html lang="en" {
			(head)

			body {
				(main)
			}
		}
	))
}


fn full<'s, 'p, 'html>(
	sack: &'s Sack,
	main: impl Renderable + 'p,
	title: String,
) -> Result<impl Renderable + 'html, String>
where
	's: 'html,
	'p: 'html,
{
	let main = maud_move!(
		(navbar())
		(main)
	);

	bare(sack, main, title, None)
}

fn page<'s, 'p, 'html>(
	sack: &'s Sack,
	main: impl Renderable + 'p,
	title: String,
	js: Option<&'s [String]>,
) -> Result<impl Renderable + 'html, String>
where
	's: 'html,
	'p: 'html,
{
	let main = maud_move!(
		(navbar())
		(main)
		(footer(sack))
	);

	bare(sack, main, title, js)
}

pub(crate) fn to_list(sack: &Sack, list: Vec<LinkDate>, title: String) -> String {
	let mut groups = HashMap::<i32, Vec<_>>::new();

	for page in list {
		groups.entry(page.date.year()).or_default().push(page);
	}

	let mut groups: Vec<_> = groups
		.into_iter()
		.map(|(k, mut v)| {
			v.sort_by(|a, b| b.date.cmp(&a.date));
			(k, v)
		})
		.collect();

	groups.sort_by(|a, b| b.0.cmp(&a.0));

	list::list(sack, &groups, title).unwrap().render().into()
}

pub(crate) fn map<'s, 'html>(sack: &'s Sack) -> Result<impl Renderable + 'html, String>
where
	's: 'html,
{
	full(
		sack,
		maud!(
			div #map style="height: 100%; width: 100%" {}

			script type="module" {
				(Raw("import 'photos';"))
			}
		),
		String::from("Map"),
	)
}

pub(crate) fn search<'s, 'html>(sack: &'s Sack) -> String {
	page(
		sack,
		maud!(
			main #app {}
		),
		String::from("Search"),
		Some(&["search".into()])
	)
	    .unwrap()
	    .render()
	    .into()
}


/// Represents a simple post.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Flox {
	pub(crate) title: String,
	#[serde(with = "isodate")]
	pub(crate) date: DateTime<Utc>,
	pub(crate) desc: Option<String>,
}

impl Content for Flox {
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
		flox(&self.title, parsed, sack, outline, bibliography)
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

pub(crate) fn flox<'p, 's, 'html>(
	title: &str,
	parsed: &str,
	sack: &Sack,
	outline: Outline,
	bibliography: Bibliography,
) -> String {
	page(
		sack,
		maud_move!(
			main {
                div .flox-playground {
                    div .cell {
                        header {
                            h2 { "Flox" }
                        }
                        div .editor {
                            div #editor {}
                            button #run .run { "Run!" }
                        }
                    }
                    div .cell {
                        h2 { "Output" }
                        pre #output {}
                    }
                }
                (article(title, parsed, sack, outline, bibliography))
            }
		),
		String::from("Flox"),
		Some(&["editor".into()])
	)
	    .unwrap()
	    .render()
		.into()
}
