use std::collections::HashMap;
use std::fs;
use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Datelike;
use clap::{Parser, ValueEnum};
use gen::{Asset, AssetKind, Content, FileItemKind, Output, PipelineItem, Sack};
use hayagriva::Library;
use html::{Link, LinkDate, Linkable};
use hypertext::{Raw, Renderable};
use once_cell::sync::Lazy;
use serde::Deserialize;
use text::md::Outline;

use crate::gen::Dynamic;
use crate::build::build_styles;

mod md;
mod html;
mod ts;
mod gen;
mod utils;
mod text;
mod watch;
mod build;


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
        link: "https://github.com/kamoshi/kamoshi.org".into(),
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


impl Content for md::Post {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        outline: Outline,
        sack: &'s Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        html::post(self, content, outline, bib, sack)
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

    fn render(data: &str, lib: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
        text::md::parse(data, lib)
    }
}

impl Content for md::Slide {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        _: Outline,
        _: &'s Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        html::show(self, content)
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

    fn render(data: &str, _: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
        let html = data
            .split("\n-----\n")
            .map(|chunk| chunk.split("\n---\n").map(|s| text::md::parse(s, None)).map(|e| e.1).collect::<Vec<_>>())
            .map(|stack| match stack.len() > 1 {
                true  => format!("<section>{}</section>", stack.into_iter().map(|slide| format!("<section>{slide}</section>")).collect::<String>()),
                false => format!("<section>{}</section>", stack[0])
            })
            .collect::<String>();
        (Outline(vec![]), html, None)
    }
}

impl Content for md::Wiki {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        outline: Outline,
        sack: &'s Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        html::wiki(self, content, outline, sack, bib)
    }

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
        Some(Linkable::Link(Link {
            path,
            name: self.title.to_owned(),
            desc: None,
        }))
    }

    fn render(data: &str, lib: Option<&Library>) -> (Outline, String, Option<Vec<String>>) {
        text::md::parse(data, lib)
    }
}


fn to_list(list: Vec<LinkDate>) -> String {
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

    html::list("", &groups).render().into()
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
                let (outline, html, bib) = T::render(&md, lib);
                T::transform(&fm, Raw(html), outline, sack, bib).render().into()
            };

            Output {
                kind: Asset {
                    kind: gen::AssetKind::Html(Box::new(call)),
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
        gen::gather("content/about.md", &["md"].into())
            .into_iter()
            .map(to_index::<md::Post> as fn(PipelineItem) -> PipelineItem),
        gen::gather("content/posts/**/*", &["md", "mdx"].into())
            .into_iter()
            .map(to_index::<md::Post>),
        gen::gather("content/slides/**/*", &["md", "lhs"].into())
            .into_iter()
            .map(to_index::<md::Slide>),
        gen::gather("content/wiki/**/*", &["md"].into())
            .into_iter()
            .map(to_index::<md::Wiki>),
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
                kind: Dynamic::new(|_| html::map().render().to_owned().into()).into(),
                path: "map/index.html".into(),
                link: None,
            },
            Output {
                kind: Dynamic::new(|_| html::search().render().to_owned().into()).into(),
                path: "search/index.html".into(),
                link: None,
            },
            Output {
                kind: Asset {
                    kind: gen::AssetKind::Html(Box::new(|_| {
                        let data = std::fs::read_to_string("content/index.md").unwrap();
                        let (_, html, _) = text::md::parse(&data, None);
                        html::home(Raw(html)).render().to_owned().into()
                    })).into(),
                    meta: gen::FileItem {
                        kind: gen::FileItemKind::Index,
                        path: "content/index.md".into()
                    }
                }.into(),
                path: "index.html".into(),
                link: None,
            }.into(),
            Output {
                kind: Dynamic::new(|sack| to_list(sack.get_links("posts/**/*.html"))).into(),
                path: "posts/index.html".into(),
                link: None,
            },
            Output {
                kind: Dynamic::new(|sack| to_list(sack.get_links("slides/**/*.html"))).into(),
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
        gen::render_all(&assets);
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
        .arg("js/reveal.js")
        .arg("js/photos.ts")
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
