mod html;
mod model;
mod text;
mod ts;

use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::{Collection, Sack, Website};
use hypertext::Renderable;
use model::{Home, Post, Slideshow, Wiki};

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

pub struct Outline(pub Vec<(String, String)>);

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

	let website = Website::setup()
		.set_opts_sitemap("https://kamoshi.org")
		.add_collections([
			Collection::glob_with::<Home>("content", "index.md", ["md"]),
			Collection::glob_with::<Post>("content", "about.md", ["md"]),
			Collection::glob_with::<Post>("content", "posts/**/*", ["md", "mdx"]),
			Collection::glob_with::<Slideshow>("content", "slides/**/*", ["md", "lhs"]),
			Collection::glob_with::<Wiki>("content", "wiki/**/*", ["md"]),
			Collection::glob_with::<Post>("content", "projects/flox.md", ["md"]),
		])
		.add_styles(["styles".into()])
		.add_scripts([
			("search", "./js/search/dist/search.js"),
			("photos", "./js/vanilla/photos.js"),
			("reveal", "./js/vanilla/reveal.js"),
			("editor", "./js/flox/main.ts"),
			("lambda", "./js/flox/lambda.ts"),
		])
		// Task: generate home page
		.add_task(|sack| {
			let query = sack.get_content::<Home>("").unwrap();
			let (parsed, _, _) = text::md::parse(query.content, &sack, query.area, None);
			let out_buff = html::home(&sack, &parsed);
			vec![("index.html".into(), out_buff)]
		})
		.add_task(|sack| {
			let query = sack.get_content::<Post>("about").unwrap();
			let (parsed, outline, bib) =
				html::post::parse_content(query.content, &sack, query.area, None);
			let out_buff = html::post::as_html(query.meta, &parsed, &sack, outline, bib);
			vec![(query.slug.join("index.html"), out_buff)]
		})
		// Task: generate posts
		.add_task(|sack| {
			sack.get_content_list::<Post>("posts/**/*")
				.into_iter()
				.map(|query| {
					let bibliography = sack.get_library(query.area);
					let (parsed, outline, bib) =
						html::post::parse_content(query.content, &sack, query.area, bibliography);
					let out_buff = html::post::as_html(query.meta, &parsed, &sack, outline, bib);
					(query.slug.join("index.html"), out_buff)
				})
				.collect()
		})
		// Task: generate slides
		.add_task(|sack| {
			sack.get_content_list::<Slideshow>("slides/**/*")
				.into_iter()
				.map(|query| {
					let (parsed, outline, bib) =
						html::slideshow::parse_content(query.content, &sack, query.area, None);
					let out_buff =
						html::slideshow::as_html(query.meta, &parsed, &sack, outline, bib);
					(query.slug.join("index.html"), out_buff)
				})
				.collect()
		})
		// Task: generate wiki
		.add_task(|sack| {
			sack.get_content_list::<Wiki>("**/*")
				.into_iter()
				.map(|query| {
					let bibliography = sack.get_library(query.area);
					let (parsed, outline, bib) =
						html::wiki::parse_content(query.content, &sack, query.area, bibliography);
					let out_buff =
						html::wiki::as_html(query.meta, &parsed, &sack, query.slug, outline, bib);
					(query.slug.join("index.html"), out_buff)
				})
				.collect()
		})
		// Task: generate map
		.add_task(|sack| {
			vec![(
				"map/index.html".into(),
				crate::html::map(&sack, Some(&["photos".into()]))
					.unwrap()
					.render()
					.to_owned()
					.into(),
			)]
		})
		// Task: generate search
		.add_task(|sack| vec![("search/index.html".into(), crate::html::search(&sack))])
		.add_task(|sack| {
			let query = sack.get_content("projects/flox").unwrap();

			let (parsed, outline, bib) =
				html::post::parse_content(query.content, &sack, query.area, None);
			let out_buff = html::as_html(query.meta, &parsed, &sack, outline, bib);

			vec![(query.slug.join("index.html"), out_buff)]
		})
		// Task: generate project index
		.add_task(|sack| {
			vec![(
				"projects/index.html".into(),
				crate::html::to_list(
					&sack,
					sack.get_content_list::<Post>("projects/**/*")
						.into_iter()
						.map(|query| LinkDate {
							link: Link {
								path: Utf8Path::new("/").join(query.slug),
								name: query.meta.title.clone(),
								desc: query.meta.desc.clone(),
							},
							date: query.meta.date,
						})
						.collect(),
					"Projects".into(),
				),
			)]
		})
		// Task: generate post index
		.add_task(|sack| {
			vec![(
				"posts/index.html".into(),
				crate::html::to_list(
					&sack,
					sack.get_content_list::<Post>("posts/**/*")
						.into_iter()
						.map(|query| LinkDate {
							link: Link {
								path: Utf8Path::new("/").join(query.slug),
								name: query.meta.title.clone(),
								desc: query.meta.desc.clone(),
							},
							date: query.meta.date,
						})
						.collect(),
					"Posts".into(),
				),
			)]
		})
		// Task: generate slideshow index
		.add_task(|sack| {
			vec![(
				"slides/index.html".into(),
				crate::html::to_list(
					&sack,
					sack.get_content_list::<Slideshow>("slides/**/*")
						.into_iter()
						.map(|query| LinkDate {
							link: Link {
								path: Utf8Path::new("/").join(query.slug),
								name: query.meta.title.clone(),
								desc: query.meta.desc.clone(),
							},
							date: query.meta.date,
						})
						.collect(),
					"Slideshows".into(),
				),
			)]
		})
		.finish();

	match args.mode {
		Mode::Build => website.build(MyData::new()),
		Mode::Watch => website.watch(MyData::new()),
	}
}
