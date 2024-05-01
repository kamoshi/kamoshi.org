use std::process::Command;
use std::collections::HashMap;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Datelike;
use gen::{Asset, AssetKind, Content, PipelineItem, Sack, StaticItemKind};
use hayagriva::Library;
use html::{Link, LinkDate, Linkable};
use hypertext::{Raw, Renderable};
use once_cell::sync::Lazy;
use serde::Deserialize;
use text::md::Outline;

mod md;
mod html;
mod ts;
mod gen;
mod utils;
mod text;


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
        _: &'s Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        html::post(self, content, outline, bib)
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
        PipelineItem::Skip(meta) if matches!(meta.kind, StaticItemKind::Index) => meta,
        _ => return item,
    };

    let dir = meta.dir.strip_prefix("content").unwrap();
    match meta.ext.as_str() {
        "md" | "mdx" | "lhs" => {
            let path = dir.join("index.html");

            let data = fs::read_to_string(&meta.src).unwrap();
            let (fm, md) = md::preflight::<T>(&data);
            let link = T::as_link(&fm, Utf8Path::new("/").join(dir));

            let call = move |sack: &Sack| {
                let lib = sack.get_library();
                let (outline, html, bib) = T::render(&md, lib);
                T::transform(&fm, Raw(html), outline, sack, bib).render().into()
            };

            gen::Asset {
                kind: gen::AssetKind::Html(Box::new(call)),
                out: path,
                link,
                meta,
            }.into()
        },
        _ => meta.into(),
    }
}

fn to_bundle(item: PipelineItem) -> PipelineItem {
    let meta = match item {
        PipelineItem::Skip(meta) if matches!(meta.kind, StaticItemKind::Bundle) => meta,
        _ => return item,
    };

    let dir = meta.dir.strip_prefix("content").unwrap();
    let out = dir.join(meta.src.file_name().unwrap()).to_owned();

    match meta.ext.as_str() {
        "jpg" | "png" | "gif" => gen::Asset {
            kind: gen::AssetKind::Image,
            out,
            link: None,
            meta,
        }.into(),
        "bib" => {
            let data = fs::read_to_string(&meta.src).unwrap();
            let data = hayagriva::io::from_biblatex_str(&data).unwrap();

            Asset {
                kind: AssetKind::Bib(data),
                out,
                link: None,
                meta,
            }.into()
        },
        _ => meta.into(),
    }
}


fn main() {
    if fs::metadata("dist").is_ok() {
        println!("Cleaning dist");
        fs::remove_dir_all("dist").unwrap();
    }

    fs::create_dir("dist").unwrap();

    let assets: Vec<Asset> = vec![
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
                println!("Skipping {}", skip.src);
                None
            },
            PipelineItem::Take(take) => Some(take),
        })
        .collect();

    let assets: Vec<Vec<gen::Item>> = vec![
        assets.into_iter()
            .map(Into::into)
            .collect(),
        vec![
            gen::Virtual::new("map/index.html", |_| html::map().render().to_owned().into()).into(),
            gen::Virtual::new("search/index.html", |_| html::search().render().to_owned().into()).into(),
            gen::Asset {
                kind: gen::AssetKind::Html(Box::new(|_| {
                    let data = std::fs::read_to_string("content/index.md").unwrap();
                    let (_, html, bib) = text::md::parse(&data, None);
                    html::home(Raw(html)).render().to_owned().into()
                })),
                out: "index.html".into(),
                link: None,
                meta: gen::StaticItem {
                    kind: gen::StaticItemKind::Index,
                    ext: "md".into(),
                    dir: "".into(),
                    src: "content/index.md".into()
                }
            }.into(),
            gen::Virtual("posts/index.html".into(), Box::new(|all|
                to_list(all.get_links("posts/**/*.html"))
            )).into(),
            gen::Virtual("slides/index.html".into(), Box::new(|all|
                to_list(all.get_links("slides/**/*.html"))
            )).into(),
        ],
    ];

    let all: Vec<gen::Item> = assets
        .into_iter()
        .flatten()
        .collect();

    {
        let now = std::time::Instant::now();
        gen::render(&all);
        println!("Elapsed: {:.2?}", now.elapsed());
    }

    utils::copy_recursively(std::path::Path::new("public"), std::path::Path::new("dist")).unwrap();

    let css = grass::from_path("styles/styles.scss", &grass::Options::default()).unwrap();
    fs::write("dist/styles.css", css).unwrap();

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
