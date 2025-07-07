mod html;
mod md;
mod model;
mod pf;
mod rss;
mod ts;

use std::borrow::Cow;
use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::md::yaml;
use hauchiwa::plugin::ts::Script;
use hauchiwa::{Hook, Loader, TaskResult, ViewPage, Website};
use hayagriva::Library;
use hypertext::Renderable;
use model::{Home, Post, Project, Slideshow, Wiki};
use sequoia_openpgp::Cert;
use sequoia_openpgp::parse::Parse;

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

type Page = (Utf8PathBuf, String);

fn render_page_post(ctx: &Context, item: ViewPage<Post>) -> TaskResult<Page> {
    let pattern = format!("{}/*.bib", item.area);
    let bibtex = ctx.glob::<Bibtex>(&pattern)?;

    let parsed = crate::md::parse(item.content, ctx, item.area, bibtex.map(|x| &x.data));
    let buffer = crate::html::post::render(
        item.meta,
        &parsed.0,
        ctx,
        item.info,
        parsed.1,
        parsed.2,
        bibtex.map(|x| x.path.as_ref()),
    )?
    .render()
    .into();

    Ok((item.slug.join("index.html"), buffer))
}

fn render_page_slideshow(ctx: &Context, query: ViewPage<Slideshow>) -> Page {
    let parsed = html::slideshow::parse_content(query.content, ctx, query.area, None);
    let buffer = html::slideshow::as_html(query.meta, &parsed.0, ctx, parsed.1, parsed.2);
    (query.slug.join("index.html"), buffer)
}

fn render_page_wiki(ctx: &Context, query: ViewPage<Wiki>) -> TaskResult<Page> {
    let pattern = format!("{}/*", query.area);
    let bibtex = ctx.glob::<Bibtex>(&pattern)?;

    let parsed = crate::md::parse(query.content, ctx, query.area, bibtex.map(|x| &x.data));
    let buffer = crate::html::wiki::wiki(
        query.meta,
        &parsed.0,
        ctx,
        query.slug,
        parsed.1,
        parsed.2,
        bibtex.map(|x| x.path.as_ref()),
    );
    Ok((query.slug.join("index.html"), buffer))
}

struct Bibtex {
    path: Utf8PathBuf,
    data: Library,
}

fn main() -> ExitCode {
    /// Base path for content files
    const BASE: &str = "content";

    let args = Args::parse();

    let mut website = Website::configure()
        .set_opts_sitemap("https://kamoshi.org")
        .add_loaders([
            Loader::glob_content(BASE, "index.md", yaml::<Home>),
            Loader::glob_content(BASE, "posts/**/*.md", yaml::<Post>),
            Loader::glob_content(BASE, "slides/**/*.md", yaml::<Slideshow>),
            Loader::glob_content(BASE, "slides/**/*.lhs", yaml::<Slideshow>),
            Loader::glob_content(BASE, "wiki/**/*.md", yaml::<Wiki>),
            Loader::glob_content(BASE, "projects/**/*.md", yaml::<Project>),
            Loader::glob_content(BASE, "about/index.md", yaml::<Post>),
            // .asc -> Pubkey
            Loader::glob_content(BASE, "about/*.asc", |text| {
                let fingerprint = Cert::from_reader(text.as_bytes())?
                    .primary_key()
                    .key()
                    .fingerprint()
                    .to_spaced_hex();

                Ok((Pubkey { fingerprint }, String::from(text)))
            }),
            // twtxt.txt
            Loader::glob_content(BASE, "twtxt.txt", |text| {
                let entries = text
                    .lines()
                    .map(str::parse::<MicroblogEntry>)
                    .collect::<Result<Vec<_>, _>>()?;

                Ok((Microblog { entries }, String::from(text)))
            }),
            // .bib -> Bibtex
            Loader::glob_asset(BASE, "**/*.bib", |rt, data| {
                let path = rt.store(&data, "bib").unwrap();
                let text = String::from_utf8_lossy(&data);
                let data = hayagriva::io::from_biblatex_str(&text).unwrap();

                Bibtex { path, data }
            }),
            // images
            Loader::glob_images(BASE, "**/*.jpg"),
            Loader::glob_images(BASE, "**/*.png"),
            Loader::glob_images(BASE, "**/*.gif"),
            // stylesheets
            Loader::glob_style("styles", "**/[!_]*.scss"),
            // scripts
            Loader::glob_svelte("scripts", "src/*/App.svelte"),
            Loader::glob_scripts("scripts", "src/*/main.ts"),
        ])
        // Generate the home page.
        .add_task(|ctx| {
            let item = ctx.glob_page::<Home>("")?;
            let text = md::parse(item.content, &ctx, item.area, None).0;
            let html = html::home(&ctx, &text)?;

            Ok(vec![("index.html".into(), html)])
        })
        // Generate the about page.
        .add_task(|ctx| {
            let item = ctx.glob_page::<Post>("about")?;
            let pubkey_ident = ctx.glob_page::<Pubkey>("about/pubkey-ident")?;
            let pubkey_email = ctx.glob_page::<Pubkey>("about/pubkey-email")?;

            let (parsed, outline, _) = crate::md::parse(item.content, &ctx, item.area, None);

            let html = crate::html::about::render(
                &ctx,
                &item,
                parsed,
                outline,
                &pubkey_ident,
                &pubkey_email,
            )?
            .render()
            .into();

            let pages = vec![
                ("about/index.html".into(), html),
                ("pubkey_ident.asc".into(), pubkey_ident.content.to_owned()),
                ("pubkey_email.asc".into(), pubkey_email.content.to_owned()),
            ];

            Ok(pages)
        })
        // POSTS
        .add_task(|sack| {
            let pages = sack
                .glob_pages::<Post>("posts/**/*")?
                .into_iter()
                .map(|query| render_page_post(&sack, query))
                .collect::<Result<_, _>>()?;
            Ok(pages)
        })
        .add_task(|sack| {
            Ok(vec![(
                "posts/index.html".into(),
                crate::html::to_list(
                    &sack,
                    sack.glob_pages::<Post>("posts/**/*")?
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
            let pages = sack
                .glob_pages::<Slideshow>("slides/**/*")?
                .into_iter()
                .map(|query| render_page_slideshow(&sack, query))
                .collect();

            Ok(pages)
        })
        .add_task(|sack| {
            Ok(vec![(
                "slides/index.html".into(),
                crate::html::to_list(
                    &sack,
                    sack.glob_pages::<Slideshow>("slides/**/*")?
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
            let data = ctx.glob_pages::<Project>("projects/**/*")?;
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
            let pages = sack
                .glob_pages::<Wiki>("**/*")?
                .into_iter()
                .map(|query| render_page_wiki(&sack, query))
                .collect::<Result<_, _>>()?;

            Ok(pages)
        })
        // MAP
        .add_task(|ctx| {
            let script = ctx.get::<Script>("scripts/src/photos/main.ts")?;

            Ok(vec![(
                "map/index.html".into(),
                crate::html::map(&ctx, Cow::Borrowed(&[script.path.to_string()]))?
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
        // microblog
        .add_task(|ctx| {
            let data = ctx.glob_page::<Microblog>("twtxt")?;
            let html = html::microblog::render(&ctx, data.meta)?.render().into();

            let mut pages = vec![
                ("twtxt.txt".into(), data.content.to_owned()),
                ("thoughts/index.html".into(), html),
            ];

            for entry in &data.meta.entries {
                let html = html::microblog::render_entry(&ctx, entry)?.render().into();

                pages.push((
                    format!("thoughts/{}/index.html", entry.date.timestamp()).into(),
                    html,
                ));
            }

            Ok(pages)
        })
        .add_hook(Hook::post_build(crate::pf::build_pagefind))
        // TODO: Sitemap.xml
        .add_hook(Hook::post_build(|_| Ok(())))
        .finish();

    let res = match args.mode {
        Mode::Build => website.build(Global::new()),
        Mode::Watch => website.watch(Global::new()),
    };

    match res {
        Ok(_) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}
