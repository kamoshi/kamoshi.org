mod build;
mod html;
mod pipeline;
mod text;
mod ts;
mod utils;
mod watch;

use std::collections::{HashMap, HashSet};
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

	let sources = &[
		Source {
			path: "content/about.md",
			exts: ["md"].into(),
			func: process_content::<crate::html::Post>,
		},
		Source {
			path: "content/posts/**/*",
			exts: ["md", "mdx"].into(),
			func: process_content::<crate::html::Post>,
		},
		Source {
			path: "content/slides/**/*",
			exts: ["md", "lhs"].into(),
			func: process_content::<crate::html::Slideshow>,
		},
		Source {
			path: "content/wiki/**/*",
			exts: ["md"].into(),
			func: process_content::<crate::html::Wiki>,
		},
	];

	let special = vec![
		Output {
			kind: Virtual::new(|sack| crate::html::map(sack).render().to_owned().into()).into(),
			path: "map/index.html".into(),
			link: None,
		},
		Output {
			kind: Virtual::new(|sack| crate::html::search(sack).render().to_owned().into()).into(),
			path: "search/index.html".into(),
			link: None,
		},
		Output {
			kind: Asset {
				kind: pipeline::AssetKind::html(|sack| {
					let data = std::fs::read_to_string("content/index.md").unwrap();
					let (_, html, _) = text::md::parse(data, None, "".into(), HashMap::new());
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
				crate::html::to_list(
					sack,
					sack.get_links("slides/**/*.html"),
					"Slideshows".into(),
				)
			})
			.into(),
			path: "slides/index.html".into(),
			link: None,
		},
	];

	match args.mode {
		Mode::Build => {
			let _ = build(&ctx, sources, special);
		}
		Mode::Watch => {
			let state = build(&ctx, sources, special);
			watch::watch(&ctx, sources, state).unwrap()
		}
	}
}

#[derive(Debug)]
struct Source {
	pub path: &'static str,
	pub exts: HashSet<&'static str>,
	pub func: fn(PipelineItem) -> PipelineItem,
}

impl Source {
	fn get(&self) -> Vec<PipelineItem> {
		pipeline::gather(self.path, &self.exts)
			.into_iter()
			.map(self.func)
			.collect()
	}

	fn get_maybe(&self, path: &Utf8Path) -> Option<PipelineItem> {
		let pattern = glob::Pattern::new(self.path).expect("Bad pattern");
		if !pattern.matches_path(path.as_std_path()) {
			return None;
		};

		let item = match path.is_file() {
			true => Some(crate::pipeline::to_source(path.to_owned(), &self.exts)),
			false => None,
		};

		item.map(Into::into).map(self.func)
	}
}

fn build(ctx: &BuildContext, sources: &[Source], special: Vec<Output>) -> Vec<Output> {
	crate::build::clean_dist();

	let sources: Vec<_> = sources
		.iter()
		.flat_map(Source::get)
		.map(to_bundle)
		.filter_map(Option::from)
		.collect();

	let assets: Vec<_> = sources.iter().chain(special.iter()).collect();

	let lmao = crate::build::build_content(ctx, &assets, &assets, None);
	crate::build::build_content(ctx, &assets, &assets, Some(lmao));
	crate::build::build_static();
	crate::build::build_styles();
	crate::build::build_pagefind();
	crate::build::build_js();

	sources.into_iter().chain(special).collect()
}

pub fn parse_frontmatter<D>(raw: &str) -> (D, String)
where
	D: for<'de> Deserialize<'de>,
{
	let parser = Matter::<YAML>::new();
	let result = parser.parse_with_struct::<D>(raw).unwrap();

	(
		// Just the front matter
		result.data,
		// The rest of the content
		result.content,
	)
}

fn process_content<T>(item: PipelineItem) -> PipelineItem
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
			let raw = fs::read_to_string(&meta.path).unwrap();
			let (matter, parsed) = parse_frontmatter::<T>(&raw);
			let link = T::as_link(&matter, Utf8Path::new("/").join(&dir));

			Output {
				kind: Asset {
					kind: pipeline::AssetKind::html(move |sack| {
						let lib = sack.get_library();
						let (outline, parsed, bib) = T::parse(
							parsed.clone(),
							lib,
							dir.clone(),
							sack.hash.as_ref().map(ToOwned::to_owned).unwrap_or_default(),
						);
						T::render(matter.clone(), sack, Raw(parsed), outline, bib)
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
