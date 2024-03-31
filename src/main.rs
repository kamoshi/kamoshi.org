use std::{collections::HashMap, path::Path};
use std::fs;
use chrono::Datelike;
use grass;
use html::LinkableData;
use hypertext::{Raw, Renderable};

mod md;
mod html;
mod ts;
mod gen;
mod utils;


trait Transformable {
    fn transform<'f, 'm, 'html, T>(&'f self, content: T) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            T: Renderable + 'm;

    fn as_link(&self, path: String) -> Option<html::LinkableData>;

    fn render(data: &str) -> String;
}

impl Transformable for md::Post {
    fn transform<'f, 'm, 'html, T>(&'f self, content: T) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            T: Renderable + 'm {
        html::post(self, content)
    }

    fn as_link(&self, path: String) -> Option<html::LinkableData> {
        Some(html::LinkableData {
            path: path.strip_suffix("index.html").unwrap().to_owned(),
            name: self.title.to_owned(),
            date: self.date.to_owned(),
            desc: self.desc.to_owned(),
        })
    }

    fn render(data: &str) -> String {
        md::render(data)
    }
}

impl Transformable for md::Slide {
    fn transform<'f, 'm, 'html, T>(&'f self, content: T) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            T: Renderable + 'm {
        html::show(self, content)
    }

    fn as_link(&self, path: String) -> Option<html::LinkableData> {
        Some(html::LinkableData {
            path: path.strip_suffix("index.html").unwrap().to_owned(),
            name: self.title.to_owned(),
            date: self.date.to_owned(),
            desc: self.desc.to_owned(),
        })
    }

    fn render(data: &str) -> String {
        data
            .split("\n-----\n")
            .map(|chunk| chunk.split("\n---\n").map(md::render).collect::<Vec<_>>())
            .map(|stack| match stack.len() > 1 {
                true  => format!("<section>{}</section>", stack.into_iter().map(|slide| format!("<section>{slide}</section>")).collect::<String>()),
                false => format!("<section>{}</section>", stack[0])
            })
            .collect::<String>()
    }
}

impl Transformable for md::Wiki {
    fn transform<'f, 'm, 'html, T>(&'f self, content: T) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            T: Renderable + 'm {
        html::wiki(self, content)
    }

    fn as_link(&self, _: String) -> Option<html::LinkableData> {
        None
    }

    fn render(data: &str) -> String {
        md::render(data)
    }
}


fn to_list(list: Vec<LinkableData>) -> String {
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


fn transform<T>(meta: gen::Source) -> gen::Asset
    where
        T: for<'de> serde::Deserialize<'de>,
        T: Transformable + 'static,
{
    let loc = meta.dirs.strip_prefix("content").unwrap();

    match meta.kind {
        gen::SourceKind::Index => match meta.ext.as_str() {
            "md" | "mdx" | "lhs" => {
                let loc = loc.join("index.html");

                let data = fs::read_to_string(&meta.path).unwrap();
                let (fm, md) = md::preflight::<T>(&data);
                let link = T::as_link(&fm, Path::new("/").join(&loc).to_str().unwrap().to_owned());

                let call = move |assets: &[&gen::Asset]| {
                    // let lib = assets.iter().filter_map(|&a| match &a.kind {
                    //     gen::AssetKind::Bib(lib) => Some(lib),
                    //     _ => None,
                    // }).next();
                    //
                    // println!("{:?}", lib);


                    let data = T::render(&md);
                    let data = T::transform(&fm, Raw(data)).render().into();
                    data
                };

                gen::Asset {
                    kind: gen::AssetKind::Html(Box::new(call)),
                    out: loc,
                    link,
                    meta,
                }
            },
            _ => gen::Asset {
                kind: gen::AssetKind::Unknown,
                out: loc.join(meta.path.file_name().unwrap()).to_owned(),
                link: None,
                meta,
            }
        },
        gen::SourceKind::Asset => {
            let loc = loc.join(meta.path.file_name().unwrap()).to_owned();
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
                    let data = md::render(&data);
                    html::home(Raw(data)).render().to_owned().into()
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
            gen::Virtual("posts/index.html".into(), Box::new(|assets| {
                let pattern = glob::Pattern::new("posts/**/*.html").unwrap();
                let posts = assets.iter()
                    .filter(|f| pattern.matches_path(&f.out))
                    .filter_map(|f| f.link.clone())
                    .collect();
                to_list(posts)
            })).into(),
            gen::Virtual("slides/index.html".into(), Box::new(|assets| {
                let pattern = glob::Pattern::new("slides/**/*.html").unwrap();
                let posts = assets.iter()
                    .filter(|f| pattern.matches_path(&f.out))
                    .filter_map(|f| f.link.clone())
                    .collect();
                to_list(posts)
            })).into(),
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

    use std::process::Command;

    let res = Command::new("pagefind")
        .args(&["--site", "dist"])
        .output()
        .unwrap();

    println!("{}", String::from_utf8(res.stdout).unwrap());

    let res = Command::new("esbuild")
        .arg("js/splash.js")
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
