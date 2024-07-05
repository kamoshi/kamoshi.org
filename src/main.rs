mod build;
mod html;
mod pipeline;
mod text;
mod ts;
mod utils;
mod watch;

use std::collections::HashSet;
use std::fs;
use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use hypertext::{Raw, Renderable};
use pipeline::{Asset, AssetKind, Content, FileItemKind, Output, PipelineItem};
use serde::Deserialize;

use crate::build::build_styles;
use crate::pipeline::Virtual;

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

#[derive(Debug)]
struct BuildContext {
	pub mode: Mode,
	pub year: i32,
	pub date: String,
	pub link: String,
	pub hash: String,
}

#[derive(Debug, Clone)]
pub struct Link {
	pub path: Utf8PathBuf,
	pub name: String,
	pub desc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LinkDate {
	pub link: Link,
	pub date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum Linkable {
	Link(Link),
	Date(LinkDate),
}

fn main() {
	let args = Args::parse();
	let time = chrono::Utc::now();

	let ctx = BuildContext {
		mode: args.mode,
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
	};

	match args.mode {
		Mode::Build => {
			build(&ctx);
		}
		Mode::Watch => {
			build(&ctx);
			watch::watch().unwrap()
		}
	}
}

struct Source {
	path: &'static str,
	exts: HashSet<&'static str>,
	func: fn(PipelineItem) -> PipelineItem,
}

impl Source {
	fn get(&self) -> Vec<PipelineItem> {
		pipeline::gather(self.path, &self.exts)
			.into_iter()
			.map(self.func)
			.collect()
	}
}

fn build(ctx: &BuildContext) {
	if fs::metadata("dist").is_ok() {
		println!("Cleaning dist");
		fs::remove_dir_all("dist").unwrap();
	}

	fs::create_dir("dist").unwrap();

	let sources = vec![
		Source {
			path: "content/about.md",
			exts: ["md"].into(),
			func: as_index::<crate::html::Post>,
		},
		Source {
			path: "content/posts/**/*",
			exts: ["md", "mdx"].into(),
			func: as_index::<crate::html::Post>,
		},
		Source {
			path: "content/slides/**/*",
			exts: ["md", "lhs"].into(),
			func: as_index::<crate::html::Slideshow>,
		},
		Source {
			path: "content/wiki/**/*",
			exts: ["md"].into(),
			func: as_index::<crate::html::Wiki>,
		},
	];

	let assets: Vec<Output> = sources
		.iter()
		.flat_map(Source::get)
		.map(to_bundle)
		.filter_map(|item| match item {
			PipelineItem::Skip(skip) => {
				println!("Skipping {}", skip.path);
				None
			}
			PipelineItem::Take(take) => Some(take),
		})
		.collect();

	let assets: Vec<Output> = vec![
		assets,
		vec![
			Output {
				kind: Virtual::new(|sack| crate::html::map(sack).render().to_owned().into()).into(),
				path: "map/index.html".into(),
				link: None,
			},
			Output {
				kind: Virtual::new(|sack| crate::html::search(sack).render().to_owned().into())
					.into(),
				path: "search/index.html".into(),
				link: None,
			},
			Output {
				kind: Asset {
					kind: pipeline::AssetKind::html(|sack| {
						let data = std::fs::read_to_string("content/index.md").unwrap();
						let (_, html, _) = text::md::parse(data, None);
						crate::html::home(sack, Raw(html))
							.render()
							.to_owned()
							.into()
					}),
					meta: pipeline::FileItem {
						kind: pipeline::FileItemKind::Index,
						path: "content/index.md".into(),
					},
				}
				.into(),
				path: "index.html".into(),
				link: None,
			},
			Output {
				kind: Virtual::new(|sack| {
					crate::html::to_list(sack, sack.get_links("posts/**/*.html"), "Posts".into())
				})
				.into(),
				path: "posts/index.html".into(),
				link: None,
			},
			Output {
				kind: Virtual::new(|sack| {
					crate::html::to_list(sack, sack.get_links("slides/**/*.html"), "Slideshows".into())
				})
				.into(),
				path: "slides/index.html".into(),
				link: None,
			},
		],
	]
	.into_iter()
	.flatten()
	.collect();

	{
		let now = std::time::Instant::now();
		pipeline::render_all(ctx, &assets);
		println!("Elapsed: {:.2?}", now.elapsed());
	}

	utils::copy_recursively(std::path::Path::new("public"), std::path::Path::new("dist")).unwrap();

	build_styles();

	let res = Command::new("pagefind")
		.args(["--site", "dist"])
		.output()
		.unwrap();

	println!("{}", String::from_utf8(res.stdout).unwrap());

	let res = Command::new("esbuild")
		.arg("js/vanilla/reveal.js")
		.arg("js/vanilla/photos.ts")
		.arg("js/search/dist/search.js")
		.arg("--format=esm")
		.arg("--bundle")
		.arg("--splitting")
		.arg("--minify")
		.arg("--outdir=dist/js/")
		.output()
		.unwrap();

	println!("{}", String::from_utf8(res.stderr).unwrap());
}

pub fn parse_frontmatter<T>(raw: &str) -> (T, String)
where
	T: for<'de> Deserialize<'de>,
{
	let matter = Matter::<YAML>::new();
	let result = matter.parse(raw);

	(
		// Just the front matter
		result.data.unwrap().deserialize::<T>().unwrap(),
		// The rest of the content
		result.content,
	)
}

fn as_index<T>(item: PipelineItem) -> PipelineItem
where
	T: for<'de> Deserialize<'de> + Content + Clone + 'static,
{
	let meta = match item {
		PipelineItem::Skip(e) if matches!(e.kind, FileItemKind::Index) => e,
		_ => return item,
	};

	let dir = meta.path.parent().unwrap().strip_prefix("content").unwrap();
	let dir = match meta.path.file_stem().unwrap() {
		"index" => dir.to_owned(),
		name => dir.join(name),
	};
	let path = dir.join("index.html");

	match meta.path.extension() {
		Some("md" | "mdx" | "lhs") => {
			let data = fs::read_to_string(&meta.path).unwrap();
			let (fm, md) = parse_frontmatter::<T>(&data);
			let link = T::as_link(&fm, Utf8Path::new("/").join(dir));

			Output {
				kind: Asset {
					kind: pipeline::AssetKind::html(move |sack| {
						let lib = sack.get_library();
						let (outline, parsed, bib) = T::parse(md.clone(), lib);
						T::render(fm.clone(), sack, Raw(parsed), outline, bib)
							.render()
							.into()
					}),
					meta,
				}
				.into(),
				path,
				link,
			}
			.into()
		}
		_ => meta.into(),
	}
}

fn to_bundle(item: PipelineItem) -> PipelineItem {
	let meta = match item {
		PipelineItem::Skip(meta) if matches!(meta.kind, FileItemKind::Bundle) => meta,
		_ => return item,
	};

	let path = meta.path.strip_prefix("content").unwrap().to_owned();

	match meta.path.extension() {
		// any image
		Some("jpg" | "png" | "gif") => Output {
			kind: Asset {
				kind: AssetKind::Image,
				meta,
			}
			.into(),
			path,
			link: None,
		}
		.into(),
		// bibliography
		Some("bib") => {
			let data = fs::read_to_string(&meta.path).unwrap();
			let data = hayagriva::io::from_biblatex_str(&data).unwrap();

			Output {
				kind: Asset {
					kind: AssetKind::Bibtex(data),
					meta,
				}
				.into(),
				path,
				link: None,
			}
			.into()
		}
		_ => meta.into(),
	}
}
