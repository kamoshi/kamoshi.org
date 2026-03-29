mod datalog;
mod md;
mod model;
mod plugin;
mod props;
mod rss;
mod ts;
mod typst;
mod utils;

use std::fs;
use std::process::{Command, ExitCode};

use camino::Utf8PathBuf;
use chrono::{DateTime, Datelike, Utc};
use clap::{Parser, ValueEnum};
use hauchiwa::error::RuntimeError;
use hauchiwa::loader::image::{ImageFormat, Quality};
use hauchiwa::loader::sitemap::ChangeFrequency;
use hauchiwa::{Output, TaskContext, Website};
use hayagriva::Library;
use minijinja::Value;

use crate::plugin::about::add_about;
use crate::plugin::home::add_home;
use crate::plugin::posts::add_posts;
use crate::plugin::projects::add_projects;
use crate::plugin::slides::add_slides;
use crate::plugin::tags::add_tags;
use crate::plugin::twtxt::add_twtxt;
use crate::plugin::wiki::add_teien;
use crate::props::{PropsMap, PropsSearch};

/// Base path for content files
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

#[derive(Debug, Clone)]
struct Global {
    pub repo: hauchiwa::git::GitRepo,
    pub year: i32,
    pub date: String,
    pub link: String,
    pub hash: String,
}

impl Global {
    fn new() -> Self {
        use hauchiwa::git;

        let time = chrono::Utc::now();

        Self {
            repo: git::map(git::Options::new("main")).unwrap(),
            year: time.year(),
            date: time.format("%Y/%m/%d %H:%M").to_string(),
            link: "https://codeberg.org/kamov/kamoshi.org/src/commit/".into(),
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
type Context<'a> = TaskContext<'a, Global>;

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

#[derive(Debug, Clone)]
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

fn run() -> Result<(), RuntimeError> {
    let args = Args::parse();

    let _ = fs::remove_dir_all("./dist");

    fs::write(
        "public/static/svg/footer-dither.svg",
        utils::generate_footer_dither(4, 64, 16, 42),
    )
    .expect("Failed to write footer-dither.svg");

    let mut config = Website::<Global>::design();

    let templates = config
        .load_minijinja()
        .glob("templates/**/*.jinja")
        .glob("templates/**/*.svg")
        .root("templates")
        // .filter("svag", utils::filter_svag)
        .register()?;

    let images = config
        .load_images()
        .format(ImageFormat::Avif(Quality::Lossy(80)))
        .format(ImageFormat::WebP)
        .glob("content/**/*.jpg")
        .glob("content/**/*.png")
        .glob("content/**/*.gif")
        .register()?;

    let styles = config
        .load_css()
        .entry("styles/**/[!_]*.scss")
        .watch("styles/**/*.scss")
        .minify(true)
        .register()?;

    let scripts = config
        .load_esbuild()
        .entry("scripts/**/main.ts")
        .watch("scripts/**/*.ts")
        .minify(true)
        .register()?;

    let svelte = config
        .load_svelte::<()>()
        .entry("scripts/**/App.svelte")
        .watch("scripts/**/*.svelte")
        .register()?;

    let bibtex = config
        .task()
        .name("bibtex")
        .glob("content/**/*.bib")
        .map(|_, store, input| {
            let data = input.read()?;
            let path = store.save(&data, "bib")?;
            let text = String::from_utf8_lossy(&data);
            let data = hayagriva::io::from_biblatex_str(&text).unwrap();

            Ok(Bibtex { path, data })
        })?;

    // home
    let home = add_home(&mut config, templates, images, styles, svelte)?;

    // about
    let about = add_about(&mut config, templates, images, styles)?;

    // digital garden
    let teien = add_teien(&mut config, templates, images, styles, bibtex)?;

    // twtxt
    let twtxt = add_twtxt(&mut config, templates, styles)?;

    // posts
    let (posts_data, posts) = add_posts(&mut config, templates, images, styles, scripts, bibtex)?;

    // slides
    let slides = add_slides(&mut config, templates, images, styles, scripts)?;

    // projects
    let projects = add_projects(&mut config, templates, styles)?;

    // tags
    let tags = add_tags(&mut config, templates, posts_data, styles)?;

    // other
    let other = config
        .task()
        .name("other")
        .using((templates, styles, scripts, svelte))
        .merge(|ctx, (templates, styles, scripts, svelte)| {
            let mut pages = vec![];

            {
                let styles = &[
                    styles.get("styles/styles.scss")?,
                    styles.get("styles/photos/leaflet.scss")?,
                    styles.get("styles/layouts/map.scss")?,
                ];
                let scripts = &[scripts.get("scripts/photos/main.ts")?];

                let props = PropsMap {
                    head: crate::plugin::make_props_head(ctx, "Map".to_string(), styles, scripts)?,
                    navbar: crate::plugin::make_props_navbar(),
                };
                let tmpl = templates.get_template("map.jinja")?;
                pages.push(Output::html("map", tmpl.render(&props)?));
            }

            {
                let styles = &[
                    styles.get("styles/styles.scss")?,
                    styles.get("styles/layouts/search.scss")?,
                ];
                let component = svelte.get("scripts/search/App.svelte")?;
                let scripts = &[&component.hydration];

                let props = PropsSearch {
                    head: crate::plugin::make_props_head(
                        ctx,
                        "Search".to_string(),
                        styles,
                        scripts,
                    )?,
                    navbar: crate::plugin::make_props_navbar(),
                    footer: crate::plugin::make_props_footer(ctx),
                    content: Value::from_safe_string((component.prerender)(&())?),
                };
                let tmpl = templates.get_template("search.jinja")?;
                pages.push(Output::html("search", tmpl.render(&props)?));
            }

            Ok(pages)
        });

    config
        .use_pagefind()
        .index(home)
        .index(about)
        .index(teien)
        .index(projects)
        .index(posts)
        .index(slides)
        .index(other)
        .register();

    config
        .use_sitemap(BASE_URL)
        .add(home, ChangeFrequency::Monthly, 0.8)
        .add(about, ChangeFrequency::Monthly, 0.8)
        .add(teien, ChangeFrequency::Monthly, 0.8)
        .add(twtxt, ChangeFrequency::Monthly, 0.8)
        .add(posts, ChangeFrequency::Monthly, 0.8)
        .add(slides, ChangeFrequency::Monthly, 0.8)
        .add(projects, ChangeFrequency::Monthly, 0.8)
        .add(tags, ChangeFrequency::Monthly, 0.8)
        .add(other, ChangeFrequency::Monthly, 0.8)
        .register();

    let mut website = config.finish();

    match args.mode {
        Mode::Build => {
            website
                .build(Global::new())?
                .render_waterfall_to_file(&website, "waterfall.svg")?;
        }
        Mode::Watch => {
            website.watch(Global::new())?;
        }
    };

    Ok(())
}
