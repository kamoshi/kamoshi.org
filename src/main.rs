mod about;
mod home;
mod hook;
mod html;
mod md;
mod model;
mod posts;
mod projects;
mod rss;
mod slides;
mod ts;
mod typst;
mod wiki;

use std::borrow::Cow;
use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::loader::{Content, Script};
use hauchiwa::{Hook, Page, TaskResult, Website, WithFile, loader};
use hayagriva::Library;
use hypertext::Renderable;
use model::{Post, Slideshow, Wiki};
use sequoia_openpgp::anyhow;

use crate::model::{Microblog, MicroblogEntry};

/// Base path for content files
const CONTENT: &str = "content";
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

// convenient wrapper for `Context`
type Context<'a> = hauchiwa::Context<'a, Global>;

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

fn render_page_post(ctx: &Context, item: WithFile<Content<Post>>) -> TaskResult<Page> {
    let pattern = format!("{}/*.bib", item.file.area);
    let bibtex = ctx.glob::<Bibtex>(&pattern)?.into_iter().next();

    let parsed = crate::md::parse(
        ctx,
        &item.data.text,
        &item.file.area,
        bibtex.map(|x| &x.data),
    )?;

    let buffer = crate::html::post::render(
        &item.data.meta,
        &parsed.0,
        ctx,
        item.file.info.as_ref(),
        parsed.1,
        parsed.2,
        bibtex.map(|x| x.path.as_ref()),
        &item.data.meta.tags,
    )?
    .render()
    .into();

    Ok(Page::text(item.file.area.join("index.html"), buffer))
}

fn render_page_slideshow(
    ctx: &Context,
    item: WithFile<Content<Slideshow>>,
) -> anyhow::Result<Page> {
    let parsed = html::slideshow::parse_content(ctx, &item.data.text, &item.file.area, None)?;
    let buffer = html::slideshow::as_html(&item.data.meta, &parsed.0, ctx, parsed.1, parsed.2);
    Ok(Page::text(item.file.area.join("index.html"), buffer))
}

fn render_page_wiki(ctx: &Context, item: WithFile<Content<Wiki>>) -> TaskResult<Page> {
    let pattern = format!("{}/*", item.file.area);
    let bibtex = ctx.glob::<Bibtex>(&pattern)?.into_iter().next();

    let parsed = crate::md::parse(
        ctx,
        &item.data.text,
        &item.file.area,
        bibtex.map(|x| &x.data),
    )?;

    let buffer = crate::html::wiki::wiki(
        &item.data.meta,
        &parsed.0,
        ctx,
        &item.file.area,
        parsed.1,
        &parsed.2,
        bibtex.map(|x| x.path.as_ref()),
    );

    Ok(Page::text(item.file.area.join("index.html"), buffer))
}

struct Bibtex {
    path: Utf8PathBuf,
    data: Library,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> TaskResult<()> {
    let args = Args::parse();

    let mut website = Website::config()
        .load_git(".")?
        .add_plugins([
            home::PLUGIN,
            about::PLUGIN,
            posts::PLUGIN,
            slides::PLUGIN,
            projects::PLUGIN,
            wiki::PLUGIN,
        ])
        .add_loaders([
            // twtxt.txt
            loader::glob_assets(CONTENT, "twtxt.txt", |_, data| {
                let data = String::from_utf8_lossy(&data);
                let entries = data
                    .lines()
                    .filter(|line| {
                        let line = line.trim_start();
                        !line.is_empty() && !line.starts_with('#')
                    })
                    .map(str::parse::<MicroblogEntry>)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                Ok(Microblog {
                    entries,
                    data: data.to_string(),
                })
            }),
            // .bib -> Bibtex
            loader::glob_assets(CONTENT, "**/*.bib", |rt, data| {
                let path = rt.store(&data, "bib")?;
                let text = String::from_utf8_lossy(&data);
                let data = hayagriva::io::from_biblatex_str(&text).unwrap();

                Ok(Bibtex { path, data })
            }),
            // images
            loader::glob_images(CONTENT, "**/*.jpg"),
            loader::glob_images(CONTENT, "**/*.png"),
            loader::glob_images(CONTENT, "**/*.gif"),
            // stylesheets
            loader::glob_styles("styles", "**/[!_]*.scss"),
            // scripts
            loader::glob_svelte::<()>("scripts", "src/*/App.svelte"),
            loader::glob_scripts("scripts", "src/*/main.ts"),
            // github
            loader::async_asset("hauchiwa", async |_| {
                const URL: &str =
                    "https://raw.githubusercontent.com/kamoshi/hauchiwa/refs/heads/main/README.md";

                Ok(reqwest::get(URL).await?.text().await?)
            }),
        ])
        // MAP
        .add_task("map", |ctx| {
            let script = ctx.get::<Script>("src/photos/main.ts")?;

            Ok(vec![Page::text(
                "map/index.html".into(),
                crate::html::map(&ctx, Cow::Borrowed(&[script.path.to_string()]))?
                    .render()
                    .to_owned()
                    .into(),
            )])
        })
        // SEARCH
        .add_task("search", |sack| {
            Ok(vec![Page::text(
                "search/index.html".into(),
                crate::html::search(&sack)?,
            )])
        })
        // microblog
        .add_task("microblog", |ctx| {
            let data = ctx
                .glob::<Microblog>("twtxt.txt")?
                .into_iter()
                .next()
                .unwrap();
            let html = html::microblog::render(&ctx, data)?.render().into();

            let mut pages = vec![
                Page::text("twtxt.txt".into(), data.data.clone()),
                Page::text("thoughts/index.html".into(), html),
            ];

            for entry in &data.entries {
                let html = html::microblog::render_entry(&ctx, entry)?.render().into();

                pages.push(Page::text(
                    format!("thoughts/{}/index.html", entry.date.timestamp()).into(),
                    html,
                ));
            }

            Ok(pages)
        })
        // Tags
        .add_task("tags", |ctx| {
            use std::collections::BTreeMap;

            let posts = ctx
                .glob_with_file::<Content<Post>>("posts/**/*")?
                .into_iter()
                .filter(|item| !item.data.meta.draft)
                .collect::<Vec<_>>();

            let mut tag_map: BTreeMap<String, Vec<LinkDate>> = BTreeMap::new();

            for post in &posts {
                for tag in &post.data.meta.tags {
                    tag_map
                        .entry(tag.clone())
                        .or_default()
                        .push(LinkDate::from(post));
                }
            }

            let mut pages = Vec::new();

            // Render individual tag pages
            for (tag, links) in &tag_map {
                let path = format!("tags/{tag}/index.html");

                let data = crate::html::tags::group(links);
                let html = crate::html::tags::render_tag(&ctx, &data, tag.to_owned())?;

                pages.push(Page::text(path.into(), html.render().into()));
            }

            // Render global tag index
            // let index = crate::html::tags::tag_cloud(&ctx, &tag_map, "Tag index")?;
            // pages.push(Page::text("tags/index.html".into(), index.render().into()));

            Ok(pages)
        })
        .add_hook(Hook::post_build(crate::hook::build_pagefind))
        .add_hook(Hook::post_build(crate::hook::build_sitemap))
        .finish();

    match args.mode {
        Mode::Build => website.build(Global::new())?,
        Mode::Watch => website.watch(Global::new())?,
    };

    Ok(())
}
