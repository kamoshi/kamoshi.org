mod html;
mod md;
mod model;
mod pf;
mod rss;
mod ts;

use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::{
    Collection, Hook, Processor, QueryContent, Sack, TaskResult, Website, parse_matter_yaml,
};
use hayagriva::Library;
use hypertext::Renderable;
use model::{Home, Post, Project, Slideshow, Wiki};

const BASE_URL: &str = "https://kamoshi.org/";

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
struct Global {
    pub year: i32,
    pub date: String,
    pub link: String,
    pub hash: String,
}

impl Global {
    fn new() -> Self {
        let time = chrono::Utc::now();
        Self {
            year: time.year(),
            date: time.format("%Y/%m/%d %H:%M").to_string(),
            link: "https://git.kamoshi.org/kamov/kamoshi.org".into(),
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

type MySack<'a> = Sack<'a, Global>;

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

fn process_bibliography(bytes: &[u8]) -> Library {
    let text = String::from_utf8_lossy(bytes);
    hayagriva::io::from_biblatex_str(&text).unwrap()
}

type Page = (Utf8PathBuf, String);

fn render_page_post(sack: &Sack<Global>, item: QueryContent<Post>) -> TaskResult<Page> {
    let library = sack.get_asset_any::<Library>(item.area)?;
    let parsed = crate::md::parse(item.content, sack, item.area, library);
    let buffer =
        crate::html::post::render(item.meta, &parsed.0, sack, item.info, parsed.1, parsed.2)?
            .render()
            .into();

    Ok((item.slug.join("index.html"), buffer))
}

fn render_page_slideshow(sack: &Sack<Global>, query: QueryContent<Slideshow>) -> Page {
    let parsed = html::slideshow::parse_content(query.content, sack, query.area, None);
    let buffer = html::slideshow::as_html(query.meta, &parsed.0, sack, parsed.1, parsed.2);
    (query.slug.join("index.html"), buffer)
}

fn render_page_wiki(sack: &Sack<Global>, query: QueryContent<Wiki>) -> TaskResult<Page> {
    let library = sack.get_asset_any::<Library>(query.area)?;
    let parsed = crate::md::parse(query.content, sack, query.area, library);
    let buffer =
        crate::html::wiki::wiki(query.meta, &parsed.0, sack, query.slug, parsed.1, parsed.2);
    Ok((query.slug.join("index.html"), buffer))
}

/// Base path for content files
const BASE: &str = "content";

/// Markdown file extensions
const EXTS_MD: [&str; 3] = ["md", "mdx", "lhs"];

fn main() -> ExitCode {
    let args = Args::parse();

    let website = Website::configure()
        .set_opts_sitemap("https://kamoshi.org")
        .add_collections([
            Collection::glob_with(BASE, "index.md", EXTS_MD, parse_matter_yaml::<Home>),
            Collection::glob_with(BASE, "about.md", EXTS_MD, parse_matter_yaml::<Post>),
            Collection::glob_with(BASE, "posts/**/*", EXTS_MD, parse_matter_yaml::<Post>),
            Collection::glob_with(BASE, "slides/**/*", EXTS_MD, parse_matter_yaml::<Slideshow>),
            Collection::glob_with(BASE, "wiki/**/*", EXTS_MD, parse_matter_yaml::<Wiki>),
            Collection::glob_with(BASE, "projects/**/*", EXTS_MD, parse_matter_yaml::<Project>),
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
        // Generate the home page.
        .add_task(|ctx| {
            let item = ctx.get_content::<Home>("")?;
            let text = md::parse(item.content, &ctx, item.area, None).0;
            let html = html::home(&ctx, &text)?;

            Ok(vec![("index.html".into(), html)])
        })
        // Generate the about page.
        .add_task(|ctx| {
            let item = ctx.get_content::<Post>("about")?;
            let (parsed, outline, bib) = crate::md::parse(item.content, &ctx, item.area, None);
            let html =
                crate::html::post::render(item.meta, &parsed, &ctx, item.info, outline, bib)?
                    .render()
                    .into();
            Ok(vec![(item.slug.join("index.html"), html)])
        })
        // POSTS
        .add_task(|sack| {
            Ok(sack
                .query_content::<Post>("posts/**/*")?
                .into_iter()
                .map(|query| render_page_post(&sack, query))
                .collect::<Result<_, _>>()?)
        })
        .add_task(|sack| {
            Ok(vec![(
                "posts/index.html".into(),
                crate::html::to_list(
                    &sack,
                    sack.query_content::<Post>("posts/**/*")?
                        .into_iter()
                        .map(LinkDate::from)
                        .collect(),
                    "Posts".into(),
                    "/posts/rss.xml",
                ),
            )])
        })
        .add_task(|sack| {
            let feed = rss::generate_feed::<Post>(sack, "posts", "Kamoshi.org Posts")?;
            Ok(vec![feed])
        })
        // SLIDESHOWS
        .add_task(|sack| {
            Ok(sack
                .query_content::<Slideshow>("slides/**/*")?
                .into_iter()
                .map(|query| render_page_slideshow(&sack, query))
                .collect())
        })
        .add_task(|sack| {
            Ok(vec![(
                "slides/index.html".into(),
                crate::html::to_list(
                    &sack,
                    sack.query_content::<Slideshow>("slides/**/*")?
                        .into_iter()
                        .map(LinkDate::from)
                        .collect(),
                    "Slideshows".into(),
                    "/slides/rss.xml",
                ),
            )])
        })
        .add_task(|sack| {
            let feed = rss::generate_feed::<Slideshow>(sack, "slides", "Kamoshi.org Slides")?;
            Ok(vec![feed])
        })
        // PROJECTS
        .add_task(|ctx| {
            let data = ctx.query_content::<Project>("projects/**/*")?;
            let list = crate::html::project::render_list(&ctx, data)?;

            Ok(vec![("projects/index.html".into(), list)])
        })
        // .add_task(|sack| {
        //     let query = sack.get_content("projects/flox")?;
        //     let (parsed, outline, bib) =
        //         html::post::parse_content(query.content, &sack, query.area, None);
        //     let out_buff = html::as_html(query.meta, &parsed, &sack, outline, bib);
        //     Ok(vec![(query.slug.join("index.html"), out_buff)])
        // })
        // .add_task(|sack| {
        //     Ok(vec![(
        //         "projects/index.html".into(),
        //         crate::html::to_list(
        //             &sack,
        //             sack.query_content::<Project>("projects/**/*")?
        //                 .into_iter()
        //                 .map(LinkDate::from)
        //                 .collect(),
        //             "Projects".into(),
        //             "/projects/rss.xml",
        //         ),
        //     )])
        // })
        .add_task(|sack| {
            let feed = rss::generate_feed::<Project>(sack, "projects", "Kamoshi.org Projects")?;
            Ok(vec![feed])
        })
        // WIKI
        .add_task(|sack| {
            Ok(sack
                .query_content::<Wiki>("**/*")?
                .into_iter()
                .map(|query| render_page_wiki(&sack, query))
                .collect::<Result<_, _>>()?)
        })
        // MAP
        .add_task(|sack| {
            Ok(vec![(
                "map/index.html".into(),
                crate::html::map(&sack, Some(&["photos".into()]))?
                    .render()
                    .to_owned()
                    .into(),
            )])
        })
        // SEARCH
        .add_task(|sack| {
            Ok(vec![(
                "search/index.html".into(),
                crate::html::search(&sack),
            )])
        })
        .add_hook(Hook::post_build(crate::pf::build_pagefind))
        .finish();

    let res = match args.mode {
        Mode::Build => website.build(Global::new()),
        Mode::Watch => website.watch(Global::new()),
    };

    match res {
        Ok(_) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{}", e.to_string());
            ExitCode::from(1)
        }
    }
}
