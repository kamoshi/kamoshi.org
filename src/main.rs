mod html;
mod text;
mod ts;

use camino::Utf8Path;
use clap::{Parser, ValueEnum};
use hauchiwa::{Collection, Link, LinkDate, Processor, Website};
use html::{Post, Slideshow, Wiki};
use hypertext::Renderable;

#[derive(Parser, Debug, Clone)]
struct Args {
	#[clap(value_enum, index = 1, default_value = "build")]
	mode: Mode,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Mode {
	Build,
	Watch,
}

fn main() {
	let args = Args::parse();

	let website = Website::design()
		.add_loaders(vec![
			Collection::glob_with::<Post>(
				"content",
				"about.md",
				["md"].into(),
				Processor {
					read_content: crate::html::post::parse_content,
					to_html: crate::html::post::as_html,
					to_link: crate::html::post::as_link,
				},
			),
			Collection::glob_with::<Post>(
				"content",
				"posts/**/*",
				["md", "mdx"].into(),
				Processor {
					read_content: crate::html::post::parse_content,
					to_html: crate::html::post::as_html,
					to_link: crate::html::post::as_link,
				},
			),
			Collection::glob_with::<Slideshow>(
				"content",
				"slides/**/*",
				["md", "lhs"].into(),
				Processor {
					read_content: crate::html::slideshow::parse_content,
					to_html: crate::html::slideshow::as_html,
					to_link: crate::html::slideshow::as_link,
				},
			),
			Collection::glob_with::<Wiki>(
				"content",
				"wiki/**/*",
				["md"].into(),
				Processor {
					read_content: crate::html::wiki::parse_content,
					to_html: crate::html::wiki::as_html,
					to_link: crate::html::wiki::as_link,
				},
			),
			Collection::glob_with::<Post>(
				"content",
				"projects/flox.md",
				["md"].into(),
				Processor {
					read_content: crate::html::post::parse_content,
					to_html: crate::html::as_html,
					to_link: crate::html::post::as_link,
				},
			),
		])
		.js("search", "./js/search/dist/search.js")
		.js("photos", "./js/vanilla/photos.js")
		.js("reveal", "./js/vanilla/reveal.js")
		.js("editor", "./js/flox/main.ts")
		.js("lambda", "./js/flox/lambda.ts")
		.add_virtual(
			|sack| crate::html::map(sack).unwrap().render().to_owned().into(),
			"map/index.html".into(),
		)
		.add_virtual(|sack| crate::html::search(sack), "search/index.html".into())
		.add_virtual(
			|sack| {
				crate::html::to_list(
					sack,
					sack.get_meta::<Post>("projects/**/*.html")
						.into_iter()
						.map(|(path, meta)| LinkDate {
							link: Link {
								path: Utf8Path::new("/").join(path),
								name: meta.title.clone(),
								desc: meta.desc.clone(),
							},
							date: meta.date,
						})
						.collect(),
					"Projects".into(),
				)
			},
			"projects/index.html".into(),
		)
		.add_virtual(
			|sack| {
				crate::html::to_list(
					sack,
					sack.get_meta::<Post>("posts/**/*.html")
						.into_iter()
						.map(|(path, meta)| LinkDate {
							link: Link {
								path: Utf8Path::new("/").join(path),
								name: meta.title.clone(),
								desc: meta.desc.clone(),
							},
							date: meta.date,
						})
						.collect(),
					"Posts".into(),
				)
			},
			"posts/index.html".into(),
		)
		.add_virtual(
			|sack| {
				crate::html::to_list(
					sack,
					sack.get_meta::<Slideshow>("slides/**/*.html")
						.into_iter()
						.map(|(path, meta)| LinkDate {
							link: Link {
								path: Utf8Path::new("/").join(path),
								name: meta.title.clone(),
								desc: meta.desc.clone(),
							},
							date: meta.date,
						})
						.collect(),
					"Slideshows".into(),
				)
			},
			"slides/index.html".into(),
		)
		.add_virtual(
			|sack| {
				let data = std::fs::read_to_string("content/index.md").unwrap();
				let (parsed, _, _) = text::md::parse(&data, sack, "".into(), None);
				crate::html::home(sack, &parsed)
			},
			"index.html".into(),
		)
		.finish();

	match args.mode {
		Mode::Build => website.build(),
		Mode::Watch => website.watch(),
	}
}
