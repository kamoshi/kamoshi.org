mod home;
mod isodate;
mod list;
mod misc;
mod post;
mod slideshow;
mod special;
mod wiki;

use std::collections::HashMap;

use camino::Utf8Path;
use chrono::Datelike;
use hypertext::{html_elements, maud, maud_move, GlobalAttributes, Raw, Renderable};

pub(crate) use home::home;
pub(crate) use post::Post;
pub(crate) use slideshow::Slideshow;
pub(crate) use wiki::Wiki;

use crate::{pipeline::Sack, Mode};

const JS_RELOAD: &str = r#"
const socket = new WebSocket("ws://localhost:1337");
socket.addEventListener("message", (event) => {
	console.log(event);
	window.location.reload();
});
"#;

const JS_IMPORTS: &str = r#"
{
	"imports": {
		"reveal": "/js/vanilla/reveal.js",
		"photos": "/js/vanilla/photos.js"
	}
}
"#;

fn head<'s, 'html>(sack: &'s Sack, title: String) -> impl Renderable + 'html
where
	's: 'html,
{
	let title = format!("{} | kamoshi.org", title);

	maud_move!(
		meta charset="utf-8";
		meta name="viewport" content="width=device-width, initial-scale=1";
		title {
			(title)
		}

		// link rel="sitemap" href="/sitemap.xml";

		link rel="stylesheet" href="/styles.css";
		link rel="stylesheet" href="/static/css/reveal.css";
		link rel="stylesheet" href="/static/css/leaflet.css";
		link rel="stylesheet" href="/static/css/MarkerCluster.css";
		link rel="stylesheet" href="/static/css/MarkerCluster.Default.css";
		link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
		link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
		link rel="icon" href="/favicon.ico" sizes="any";

		script type="importmap" {(Raw(JS_IMPORTS))}

		@if matches!(sack.ctx.mode, Mode::Watch) {
			script { (Raw(JS_RELOAD)) }
		}
	)
}

fn navbar() -> impl Renderable {
	static ITEMS: &[(&str, &str)] = &[
		("Posts", "/posts/"),
		("Slides", "/slides/"),
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
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	maud_move!(
		(Raw("<!DOCTYPE html>"))
		html lang="en" {
			(head(sack, title))

			body {
				(main)
			}
		}
	)
}

fn page<'s, 'p, 'html>(
	sack: &'s Sack,
	main: impl Renderable + 'p,
	title: String,
) -> impl Renderable + 'html
where
	's: 'html,
	'p: 'html,
{
	maud_move!(
		(Raw("<!DOCTYPE html>"))
		html lang="en" {
			(head(sack, title))

			body {
				(navbar())
				(main)
				(footer(sack))
			}
		}
	)
}

pub(crate) fn to_list(sack: &Sack, list: Vec<crate::LinkDate>, title: String) -> String {
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

	list::list(sack, &groups, title).render().into()
}

pub(crate) fn map<'s, 'html>(sack: &'s Sack) -> impl Renderable + 'html
where
	's: 'html,
{
	page(
		sack,
		maud!(
			main {
				div #map style="height: 100%; width: 100%" {}

				script type="module" {
					(Raw("import 'photos';"))
				}
			}
		),
		String::from("Map"),
	)
}

pub(crate) fn search<'s, 'html>(sack: &'s Sack) -> impl Renderable + 'html
where
	's: 'html,
{
	page(
		sack,
		maud!(
			main #app {}
			script type="module" src="/js/search/dist/search.js" {}
		),
		String::from("Search"),
	)
}
