mod hook;
mod html;
mod md;
mod model;
mod rss;
mod ts;
mod typst;

use std::borrow::Cow;
use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::loader::{Content, Script, yaml};
use hauchiwa::{Hook, Page, TaskResult, Website, WithFile, loader};
use hayagriva::Library;
use hypertext::Renderable;
use model::{Home, Post, Project, Slideshow, Wiki};
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::{Cert, anyhow};

use crate::model::{Microblog, MicroblogEntry, Pubkey};

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

    Ok(Page::text(item.file.slug.join("index.html"), buffer))
}

fn render_page_slideshow(
    ctx: &Context,
    item: WithFile<Content<Slideshow>>,
) -> anyhow::Result<Page> {
    let parsed = html::slideshow::parse_content(ctx, &item.data.text, &item.file.area, None)?;
    let buffer = html::slideshow::as_html(&item.data.meta, &parsed.0, ctx, parsed.1, parsed.2);
    Ok(Page::text(item.file.slug.join("index.html"), buffer))
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
        &item.file.slug,
        parsed.1,
        &parsed.2,
        bibtex.map(|x| x.path.as_ref()),
    );
    Ok(Page::text(item.file.slug.join("index.html"), buffer))
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
    /// Base path for content files
    const BASE: &str = "content";

    let args = Args::parse();

    let mut website = Website::config()
        .load_git(".")?
        .add_loaders([
            loader::glob_content(BASE, "index.md", yaml::<Home>),
            loader::glob_content(BASE, "posts/**/*.md", yaml::<Post>),
            loader::glob_content(BASE, "slides/**/*.md", yaml::<Slideshow>),
            loader::glob_content(BASE, "slides/**/*.lhs", yaml::<Slideshow>),
            loader::glob_content(BASE, "wiki/**/*.md", yaml::<Wiki>),
            loader::glob_content(BASE, "projects/**/*.md", yaml::<Project>),
            loader::glob_content(BASE, "about/index.md", yaml::<Post>),
            // .asc -> Pubkey
            loader::glob_assets(BASE, "about/*.asc", |_, data| {
                Ok(Pubkey {
                    fingerprint: Cert::from_reader(data.as_slice())?
                        .primary_key()
                        .key()
                        .fingerprint()
                        .to_spaced_hex(),
                    data: String::from_utf8(data)?.to_string(),
                })
            }),
            // twtxt.txt
            loader::glob_assets(BASE, "twtxt.txt", |_, data| {
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
            loader::glob_assets(BASE, "**/*.bib", |rt, data| {
                let path = rt.store(&data, "bib")?;
                let text = String::from_utf8_lossy(&data);
                let data = hayagriva::io::from_biblatex_str(&text).unwrap();

                Ok(Bibtex { path, data })
            }),
            // images
            loader::glob_images(BASE, "**/*.jpg"),
            loader::glob_images(BASE, "**/*.png"),
            loader::glob_images(BASE, "**/*.gif"),
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
        // Generate the home page.
        .add_task("home", |ctx| {
            let item = ctx.glob_one_with_file::<Content<Home>>("")?;
            let text = md::parse(&ctx, &item.data.text, &item.file.area, None)?.0;
            let html = html::home(&ctx, &text)?;
            Ok(vec![Page::text("index.html".into(), html)])
        })
        // Generate the about page.
        .add_task("about", |ctx| {
            let item = ctx.glob_one_with_file::<Content<Post>>("about")?;
            let pubkey_ident = ctx.get::<Pubkey>("content/about/pubkey-ident.asc")?;
            let pubkey_email = ctx.get::<Pubkey>("content/about/pubkey-email.asc")?;

            let (parsed, outline, _) =
                crate::md::parse(&ctx, &item.data.text, &item.file.area, None)?;
            let html = crate::html::about::render(
                &ctx,
                &item,
                &parsed,
                &outline,
                pubkey_ident,
                pubkey_email,
            )?
            .render()
            .into();

            let pages = vec![
                Page::text("about/index.html".into(), html),
                Page::text("pubkey_ident.asc".into(), pubkey_ident.data.to_owned()),
                Page::text("pubkey_email.asc".into(), pubkey_email.data.to_owned()),
            ];

            Ok(pages)
        })
        // Posts
        // -----
        .add_task("posts", |ctx| {
            let pages = ctx
                .glob_with_file::<Content<Post>>("posts/**/*")?
                .into_iter()
                .filter(|item| !item.data.meta.draft)
                .map(|query| render_page_post(&ctx, query))
                .collect::<Result<_, _>>()?;
            Ok(pages)
        })
        .add_task("posts_list", |ctx| {
            Ok(vec![Page::text(
                "posts/index.html".into(),
                crate::html::to_list(
                    &ctx,
                    ctx.glob_with_file::<Content<Post>>("posts/**/*")?
                        .iter()
                        .filter(|item| !item.data.meta.draft)
                        .map(LinkDate::from)
                        .collect(),
                    "Posts".into(),
                    "/posts/rss.xml",
                ),
            )])
        })
        .add_task("posts_feed", |sack| {
            let feed = rss::generate_feed::<Content<Post>>(sack, "posts", "Kamoshi.org Posts")?;
            Ok(vec![feed])
        })
        // SLIDESHOWS
        .add_task("slides", |sack| {
            let pages = sack
                .glob_with_file::<Content<Slideshow>>("slides/**/*")?
                .into_iter()
                .map(|query| render_page_slideshow(&sack, query))
                .collect::<Result<_, _>>()?;
            Ok(pages)
        })
        .add_task("slides_list", |sack| {
            Ok(vec![Page::text(
                "slides/index.html".into(),
                crate::html::to_list(
                    &sack,
                    sack.glob_with_file::<Content<Slideshow>>("slides/**/*")?
                        .into_iter()
                        .map(LinkDate::from)
                        .collect(),
                    "Slideshows".into(),
                    "/slides/rss.xml",
                ),
            )])
        })
        .add_task("slides_feed", |sack| {
            let feed =
                rss::generate_feed::<Content<Slideshow>>(sack, "slides", "Kamoshi.org Slides")?;
            Ok(vec![feed])
        })
        // PROJECTS
        .add_task("projects", |ctx| {
            let mut pages = vec![];

            let data = ctx.glob_with_file::<Content<Project>>("projects/**/*")?;
            let list = crate::html::project::render_list(&ctx, data)?;
            pages.push(Page::text("projects/index.html".into(), list));

            let text = ctx.get::<String>("hauchiwa")?;
            let (text, outline, _) = crate::md::parse(&ctx, text, "".into(), None)?;
            let html = html::project::render_page(&ctx, &text, outline)?;
            pages.push(Page::text("projects/hauchiwa/index.html".into(), html));

            Ok(pages)
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
        .add_task("projects_feed", |sack| {
            let feed =
                rss::generate_feed::<Content<Project>>(sack, "projects", "Kamoshi.org Projects")?;
            Ok(vec![feed])
        })
        // WIKI
        .add_task("wiki", |sack| {
            let pages = sack
                .glob_with_file::<Content<Wiki>>("**/*")?
                .into_iter()
                .map(|query| render_page_wiki(&sack, query))
                .collect::<Result<_, _>>()?;

            Ok(pages)
        })
        // MAP
        .add_task("map", |ctx| {
            let script = ctx.get::<Script>("scripts/src/photos/main.ts")?;

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
                .glob::<Microblog>("content/twtxt.txt")?
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
