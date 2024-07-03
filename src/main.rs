mod build;
mod html;
mod md;
mod pipeline;
mod text;
mod ts;
mod utils;
mod watch;

use std::fs;
use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use pipeline::{Asset, AssetKind, Content, FileItemKind, Output, PipelineItem, Sack};
use hypertext::{Raw, Renderable};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::pipeline::Virtual;
use crate::build::build_styles;

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
struct BuildInfo {
    pub year: i32,
    pub date: String,
    pub link: String,
    pub hash: String,
}


static REPO: Lazy<BuildInfo> = Lazy::new(|| {
    let time = chrono::Utc::now();

    BuildInfo {
        year: time.year(),
        date: time.format("%Y/%m/%d %H:%M").to_string(),
        link: "https://git.kamoshi.org/kamov/website".into(),
        hash: String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .unwrap()
                .stdout
        )
            .unwrap()
            .trim()
            .into()
    }
});


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


fn to_index<T>(item: PipelineItem) -> PipelineItem
    where
        T: for<'de> Deserialize<'de> + Content + 'static,
{
    let meta = match item {
        PipelineItem::Skip(meta) if matches!(meta.kind, FileItemKind::Index) => meta,
        _ => return item,
    };

    let dir = meta.path.parent().unwrap().strip_prefix("content").unwrap();
    let dir = match meta.path.file_stem().unwrap() {
        "index" => dir.to_owned(),
        name    => dir.join(name),
    };
    let path = dir.join("index.html");

    match meta.path.extension() {
        Some("md" | "mdx" | "lhs") => {
            let data = fs::read_to_string(&meta.path).unwrap();
            let (fm, md) = md::preflight::<T>(&data);
            let link = T::as_link(&fm, Utf8Path::new("/").join(dir));

            let call = move |sack: &Sack| {
                let lib = sack.get_library();
                let (outline, html, bib) = T::parse(&md, lib);
                T::transform(&fm, Raw(html), outline, sack, bib).render().into()
            };

            Output {
                kind: Asset {
                    kind: pipeline::AssetKind::Html(Box::new(call)),
                    meta,
                }.into(),
                path,
                link,
            }.into()
        },
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
        Some("jpg" | "png" | "gif") => {
            Output {
                kind: Asset {
                    kind: AssetKind::Image,
                    meta,
                }.into(),
                path,
                link: None,
            }.into()
        },
        // bibliography
        Some("bib") => {
            let data = fs::read_to_string(&meta.path).unwrap();
            let data = hayagriva::io::from_biblatex_str(&data).unwrap();

            Output {
                kind: Asset {
                    kind: AssetKind::Bibtex(data),
                    meta,
                }.into(),
                path,
                link: None,
            }.into()
        },
        _ => meta.into(),
    }
}


fn build() {
    if fs::metadata("dist").is_ok() {
        println!("Cleaning dist");
        fs::remove_dir_all("dist").unwrap();
    }

    fs::create_dir("dist").unwrap();

    let assets: Vec<Output> = [
        pipeline::gather("content/about.md", &["md"].into())
            .into_iter()
            .map(to_index::<crate::html::Post> as fn(PipelineItem) -> PipelineItem),
        pipeline::gather("content/posts/**/*", &["md", "mdx"].into())
            .into_iter()
            .map(to_index::<crate::html::Post>),
        pipeline::gather("content/slides/**/*", &["md", "lhs"].into())
            .into_iter()
            .map(to_index::<crate::html::Slideshow>),
        pipeline::gather("content/wiki/**/*", &["md"].into())
            .into_iter()
            .map(to_index::<crate::html::Wiki>),
    ]
        .into_iter()
        .flatten()
        .map(to_bundle)
        .filter_map(|item| match item {
            PipelineItem::Skip(skip) => {
                println!("Skipping {}", skip.path);
                None
            },
            PipelineItem::Take(take) => Some(take),
        })
        .collect();

    let assets: Vec<Output> = vec![
        assets,
        vec![
            Output {
                kind: Virtual::new(|_| crate::html::map().render().to_owned().into()).into(),
                path: "map/index.html".into(),
                link: None,
            },
            Output {
                kind: Virtual::new(|_| crate::html::search().render().to_owned().into()).into(),
                path: "search/index.html".into(),
                link: None,
            },
            Output {
                kind: Asset {
                    kind: pipeline::AssetKind::Html(Box::new(|_| {
                        let data = std::fs::read_to_string("content/index.md").unwrap();
                        let (_, html, _) = text::md::parse(&data, None);
                        crate::html::home(Raw(html)).render().to_owned().into()
                    })),
                    meta: pipeline::FileItem {
                        kind: pipeline::FileItemKind::Index,
                        path: "content/index.md".into()
                    }
                }.into(),
                path: "index.html".into(),
                link: None,
            },
            Output {
                kind: Virtual::new(|sack| crate::html::to_list(sack.get_links("posts/**/*.html"))).into(),
                path: "posts/index.html".into(),
                link: None,
            },
            Output {
                kind: Virtual::new(|sack| crate::html::to_list(sack.get_links("slides/**/*.html"))).into(),
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
        pipeline::render_all(&assets);
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

fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::Build => build(),
        Mode::Watch => {
            build();
            watch::watch().unwrap()
        },
    }
}
