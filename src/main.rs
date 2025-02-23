mod html;
mod model;
mod text;
mod ts;

use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::{Collection, Processor, QueryContent, Sack, Website, parse_matter_yaml};
use hayagriva::Library;
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

pub struct Bibliography(pub Option<Vec<String>>);
pub struct Outline(pub Vec<(String, String)>);

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

fn process_bibliography(content: &str) -> Library {
	hayagriva::io::from_biblatex_str(content).unwrap()
}

type Page = (Utf8PathBuf, String);

fn render_page_post(sack: &Sack<MyData>, query: QueryContent<Post>) -> Page {
	let library = sack.get_asset::<Library>(query.area);
	let parsed = html::post::parse_content(query.content, sack, query.area, library);
	let buffer = html::post::as_html(query.meta, &parsed.0, sack, parsed.1, parsed.2);
	(query.slug.join("index.html"), buffer)
}

fn render_page_slideshow(sack: &Sack<MyData>, query: QueryContent<Slideshow>) -> Page {
	let parsed = html::slideshow::parse_content(query.content, sack, query.area, None);
	let buffer = html::slideshow::as_html(query.meta, &parsed.0, sack, parsed.1, parsed.2);
	(query.slug.join("index.html"), buffer)
}

fn render_page_wiki(sack: &Sack<MyData>, query: QueryContent<Wiki>) -> Page {
	let library = sack.get_asset::<Library>(query.area);
	let parsed = html::wiki::parse_content(query.content, sack, query.area, library);
	let buffer = html::wiki::as_html(query.meta, &parsed.0, sack, query.slug, parsed.1, parsed.2);
	(query.slug.join("index.html"), buffer)
}

/// Base path for content files
const BASE: &str = "content";

/// Markdown file extensions
const EXTS_MD: [&str; 3] = ["md", "mdx", "lhs"];

fn main() {
	let args = Args::parse();

	let website = Website::setup()
		.set_opts_sitemap("https://kamoshi.org")
		.add_collections([
			Collection::glob_with(BASE, "index.md", EXTS_MD, parse_matter_yaml::<Home>),
			Collection::glob_with(BASE, "about.md", EXTS_MD, parse_matter_yaml::<Post>),
			Collection::glob_with(BASE, "posts/**/*", EXTS_MD, parse_matter_yaml::<Post>),
			Collection::glob_with(BASE, "slides/**/*", EXTS_MD, parse_matter_yaml::<Slideshow>),
			Collection::glob_with(BASE, "wiki/**/*", EXTS_MD, parse_matter_yaml::<Wiki>),
			Collection::glob_with(BASE, "projects/flox.md", EXTS_MD, parse_matter_yaml::<Post>),
		])
		.add_processors([
			Processor::process_images(["jpg", "png", "gif"]),
			Processor::process_assets(["bib"], process_bibliography),
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
			sack.query_content::<Post>("posts/**/*")
				.into_iter()
				.map(|query| render_page_post(&sack, query))
				.collect()
		})
		// Task: generate slides
		.add_task(|sack| {
			sack.query_content::<Slideshow>("slides/**/*")
				.into_iter()
				.map(|query| render_page_slideshow(&sack, query))
				.collect()
		})
		// Task: generate wiki
		.add_task(|sack| {
			sack.query_content::<Wiki>("**/*")
				.into_iter()
				.map(|query| render_page_wiki(&sack, query))
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
					sack.query_content::<Post>("projects/**/*")
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
					sack.query_content::<Post>("posts/**/*")
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
					sack.query_content::<Slideshow>("slides/**/*")
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
