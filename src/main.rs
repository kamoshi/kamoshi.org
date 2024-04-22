use std::process::Command;
use std::collections::HashMap;
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Datelike;
use gen::{Asset, Sack, Content};
use html::{Link, LinkDate, Linkable};
use hypertext::{Raw, Renderable};
use once_cell::sync::Lazy;
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
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        html::post(self, content, outline)
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

    fn render(data: &str) -> (Outline, String) {
        text::md::parse(data)
    }
}

impl Content for md::Slide {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        _: Outline,
        _: &'s Sack,
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

    fn render(data: &str) -> (Outline, String) {
        let html = data
            .split("\n-----\n")
            .map(|chunk| chunk.split("\n---\n").map(text::md::parse).map(|e| e.1).collect::<Vec<_>>())
            .map(|stack| match stack.len() > 1 {
                true  => format!("<section>{}</section>", stack.into_iter().map(|slide| format!("<section>{slide}</section>")).collect::<String>()),
                false => format!("<section>{}</section>", stack[0])
            })
            .collect::<String>();
        (Outline(vec![]), html)
    }
}

impl Content for md::Wiki {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        outline: Outline,
        sack: &'s Sack,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm {
        html::wiki(self, content, outline, sack)
    }

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable> {
        Some(Linkable::Link(Link {
            path,
            name: self.title.to_owned(),
            desc: None,
        }))
    }

    fn render(data: &str) -> (Outline, String) {
        text::md::parse(data)
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


fn transform<T>(meta: gen::Source) -> Asset
    where
        T: for<'de> serde::Deserialize<'de>,
        T: Content + 'static,
{
    let dir = meta.dirs.strip_prefix("content").unwrap();

    match meta.kind {
        gen::SourceKind::Index => match meta.ext.as_str() {
            "md" | "mdx" | "lhs" => {
                let path = dir.join("index.html");

                let data = fs::read_to_string(&meta.path).unwrap();
                let (fm, md) = md::preflight::<T>(&data);
                let link = T::as_link(&fm, Utf8Path::new("/").join(dir));

                let call = move |everything: &Sack| {
                    let (outline, html) = T::render(&md);
                    T::transform(&fm, Raw(html), outline, everything).render().into()
                };

                gen::Asset {
                    kind: gen::AssetKind::Html(Box::new(call)),
                    out: path,
                    link,
                    meta,
                }
            },
            _ => gen::Asset {
                kind: gen::AssetKind::Unknown,
                out: dir.join(meta.path.file_name().unwrap()).to_owned(),
                link: None,
                meta,
            }
        },
        gen::SourceKind::Asset => {
            let loc = dir.join(meta.path.file_name().unwrap()).to_owned();
            match meta.ext.as_str() {
                "jpg" | "png" | "gif" => gen::Asset {
                    kind: gen::AssetKind::Image,
                    out: loc,
                    link: None,
                    meta,
                },
                "bib" => {
                    let data = fs::read_to_string(&meta.path).unwrap();
                    let data = hayagriva::io::from_biblatex_str(&data).unwrap();

                    gen::Asset {
                        kind: gen::AssetKind::Bib(data),
                        out: loc,
                        link: None,
                        meta,
                    }
                },
                _ => gen::Asset {
                    kind: gen::AssetKind::Unknown,
                    out: loc,
                    link: None,
                    meta,
                },
            }
        }
    }
}

fn main() {
    if fs::metadata("dist").is_ok() {
        println!("Cleaning dist");
        fs::remove_dir_all("dist").unwrap();
    }

    fs::create_dir("dist").unwrap();

    let assets: Vec<Vec<gen::Item>> = vec![
        vec![
            gen::Virtual::new("map/index.html", |_| html::map().render().to_owned().into()).into(),
            gen::Virtual::new("search/index.html", |_| html::search().render().to_owned().into()).into(),
            gen::Asset {
                kind: gen::AssetKind::Html(Box::new(|_| {
                    let data = std::fs::read_to_string("content/index.md").unwrap();
                    let (_, html) = text::md::parse(&data);
                    html::home(Raw(html)).render().to_owned().into()
                })),
                out: "index.html".into(),
                link: None,
                meta: gen::Source {
                    kind: gen::SourceKind::Index,
                    ext: "md".into(),
                    dirs: "".into(),
                    path: "content/index.md".into()
                }
            }.into(),
            gen::Virtual("posts/index.html".into(), Box::new(|all|
                to_list(all.get_links("posts/**/*.html"))
            )).into(),
            gen::Virtual("slides/index.html".into(), Box::new(|all|
                to_list(all.get_links("slides/**/*.html"))
            )).into(),
        ],
        gen::gather("content/about.md", &["md"].into())
            .into_iter()
            .map(transform::<md::Post>)
            .map(Into::into)
            .collect(),
        gen::gather("content/posts/**/*", &["md", "mdx"].into())
            .into_iter()
            .map(transform::<md::Post>)
            .map(Into::into)
            .collect(),
        gen::gather("content/slides/**/*", &["md", "lhs"].into())
            .into_iter()
            .map(transform::<md::Slide>)
            .map(Into::into)
            .collect(),
        gen::gather("content/wiki/**/*", &["md"].into())
            .into_iter()
            .map(transform::<md::Wiki>)
            .map(Into::into)
            .collect(),
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
