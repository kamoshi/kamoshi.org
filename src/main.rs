mod html;
mod text;
mod ts;

use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::{Collection, Processor, Sack, Website};
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

#[derive(Debug, Clone)]
struct MyData {
	pub year: i32,
	pub date: String,
	pub link: String,
	pub hash: String,
}

impl MyData {
	fn new() -> Self {
		let time = chrono::Utc::now();
		Self {
			year: time.year(),
			date: time.format("%Y/%m/%d %H:%M").to_string(),
			link: "https://git.kamoshi.org/kamov/website".into(),
			hash: String::from_utf8(
				Command::new("git")
					.args(["rev-parse", "--short", "HEAD"])
					.output()
					.expect("Couldn't load git revision")
					.stdout,
			)
			.expect("Invalid UTF8")
			.trim()
			.into(),
		}
	}
}

type MySack<'a> = Sack<'a, MyData>;

#[derive(Debug, Clone)]
struct Link {
	pub path: Utf8PathBuf,
	pub name: String,
	pub desc: Option<String>,
}

#[derive(Debug, Clone)]
struct LinkDate {
	pub link: Link,
	pub date: DateTime<Utc>,
}

fn main() {
	let args = Args::parse();

	let website = Website::design()
		.add_collections(vec![
			Collection::glob_with::<Post>(
				"content",
				"about.md",
				["md"].into(),
				Processor {
					read_content: crate::html::post::parse_content,
					to_html: crate::html::post::as_html,
				},
			),
			Collection::glob_with::<Post>(
				"content",
				"posts/**/*",
				["md", "mdx"].into(),
				Processor {
					read_content: crate::html::post::parse_content,
					to_html: crate::html::post::as_html,
				},
			),
			Collection::glob_with::<Slideshow>(
				"content",
				"slides/**/*",
				["md", "lhs"].into(),
				Processor {
					read_content: crate::html::slideshow::parse_content,
					to_html: crate::html::slideshow::as_html,
				},
			),
			Collection::glob_with::<Wiki>(
				"content",
				"wiki/**/*",
				["md"].into(),
				Processor {
					read_content: crate::html::wiki::parse_content,
					to_html: crate::html::wiki::as_html,
				},
			),
			Collection::glob_with::<Post>(
				"content",
				"projects/flox.md",
				["md"].into(),
				Processor {
					read_content: crate::html::post::parse_content,
					to_html: crate::html::as_html,
				},
			),
		])
		.add_scripts(vec![
			("search", "./js/search/dist/search.js"),
			("photos", "./js/vanilla/photos.js"),
			("reveal", "./js/vanilla/reveal.js"),
			("editor", "./js/flox/main.ts"),
			("lambda", "./js/flox/lambda.ts"),
		])
		.add_virtual(
			|sack| crate::html::map(sack).unwrap().render().to_owned().into(),
			"map/index.html".into(),
		)
		.add_virtual(crate::html::search, "search/index.html".into())
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
		Mode::Build => website.build(MyData::new()),
		Mode::Watch => website.watch(MyData::new()),
	}
}
