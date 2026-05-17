use camino::Utf8Path;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::git::GitHistory;
use hauchiwa::loader::{Document, Image, Script, Stylesheet, TemplateEnv};
use hauchiwa::prelude::*;
use hypertext::prelude::*;
use minijinja::Value;

use crate::md::Parsed;
use crate::model::Post;
use crate::props::{PropsBibliography, PropsPost, PropsPostMeta, PropsPostUpdated};
use crate::{Bibtex, Context, Global, Link, LinkDate};

use super::to_list;

type PostsOutput = (Many<Document<Post>>, One<Vec<Output>>);

pub fn add_posts(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
    scripts: Many<Script>,
    bibtex: Many<Bibtex>,
) -> Result<PostsOutput, HauchiwaError> {
    let docs = config
        .load_documents::<Post>()
        .glob("content/posts/**/*.md")?
        .base("content")
        .register();

    let pages = config
        .task()
        .using((templates, docs, images, styles, scripts, bibtex))
        .merge(|ctx, (templates, docs, images, styles, scripts, bibtex)| {
            let mut pages = vec![];

            let documents = docs
                .values()
                .filter(|item| !item.matter.draft)
                .collect::<Vec<_>>();

            // render the posts
            for document in &documents {
                let bibtex = bibtex.glob(&document.meta.assets("*.bib"))?.next();

                let styles = &[
                    styles.get("styles/styles.scss")?,
                    styles.get("styles/layouts/page.scss")?,
                ];

                let mut js = vec![scripts.get("scripts/outline/main.ts")?];

                // Auto-include colocated script if present (e.g. content/posts/foo/main.ts)
                let colocated = document.meta.path.with_file_name("main.ts");
                if let Ok(script) = scripts.get(colocated.as_str()) {
                    js.push(script);
                };

                if let Some(entries) = &document.matter.scripts {
                    for entry in entries {
                        let key = format!("scripts/{}", entry);
                        js.push(scripts.get(key)?);
                    }
                }

                let parsed = crate::md::parse(
                    &document.text,
                    &document.meta,
                    None,
                    Some(&images),
                    bibtex.map(|(_, library)| &library.data),
                )?;

                let buffer = render(RenderPost {
                    ctx,
                    templates,
                    meta: &document.matter,
                    parsed,
                    info: ctx
                        .env
                        .data
                        .repo
                        .as_ref()
                        .and_then(|repo| repo.files.get(document.meta.path.as_str())),
                    library_path: bibtex.map(|(_, library)| library.path.as_path()),
                    tags: &document.matter.tags,
                    styles,
                    scripts: &js,
                })?;

                pages.push(Output::to(document).html(buffer)?);
            }

            {
                let styles = &[
                    styles.get("styles/styles.scss")?,
                    styles.get("styles/layouts/list.scss")?,
                ];

                let html = to_list(
                    ctx,
                    templates,
                    documents
                        .iter()
                        .map(|item| LinkDate {
                            link: Link {
                                path: camino::Utf8PathBuf::from(&item.meta.href),
                                name: item.matter.title.clone(),
                                desc: item.matter.desc.clone(),
                            },
                            date: item.matter.date,
                        })
                        .collect(),
                    "Posts".into(),
                    "/posts/rss.xml",
                    styles,
                )?;

                pages.push(Output::html("posts", html));
            }

            {
                pages.push(crate::rss::generate_feed(
                    &documents,
                    "posts",
                    "Kamoshi.org Posts",
                ));
            }

            Ok(pages)
        });

    Ok((docs, pages))
}

pub struct RenderPost<'a> {
    pub ctx: &'a Context<'a>,
    pub templates: &'a TemplateEnv,
    pub meta: &'a Post,
    pub parsed: Parsed,
    pub info: Option<&'a GitHistory>,
    pub library_path: Option<&'a Utf8Path>,
    pub tags: &'a [String],
    pub styles: &'a [&'a Stylesheet],
    pub scripts: &'a [&'a Script],
}

pub fn render(args: RenderPost) -> Result<String, RuntimeError> {
    let RenderPost {
        ctx,
        templates,
        meta,
        parsed,
        info,
        library_path,
        tags,
        styles,
        scripts,
    } = args;

    let outline_html = parsed.outline.render().into_inner();

    let bibliography = parsed.bibliography.map(|bib| PropsBibliography {
        items: bib.into_iter().map(Value::from_safe_string).collect(),
        library_path: library_path.map(|p| p.to_string()),
    });

    let updated = info.map(|info| {
        let info = info[0].as_ref();
        PropsPostUpdated {
            date: info.commit_date.format("%Y, %B %d").to_string(),
            date_iso: info.commit_date.format("%Y-%m-%d").to_string(),
            hash: info.abbreviated_hash.clone(),
            hash_url: format!("{}/{}", &ctx.env.data.link, &info.abbreviated_hash),
        }
    });

    let props = PropsPost {
        head: super::make_props_head(ctx, meta.title.clone(), styles, scripts)?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        title: meta.title.clone(),
        outline: Value::from_safe_string(outline_html),
        content: Value::from_safe_string(parsed.html),
        bibliography,
        metadata: PropsPostMeta {
            date_added: meta.date.format("%Y, %B %d").to_string(),
            date_added_iso: meta.date.format("%Y-%m-%d").to_string(),
            updated,
            tags: tags.to_vec(),
        },
    };

    let tmpl = templates.get_template("post.jinja")?;
    Ok(tmpl.render(&props)?)
}
