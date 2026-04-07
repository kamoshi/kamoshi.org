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

pub fn add_posts(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
    scripts: Many<Script>,
    bibtex: Many<Bibtex>,
) -> Result<(Many<Document<Post>>, One<Vec<Output>>), HauchiwaError> {
    let docs = config
        .load_documents::<Post>()
        .glob("content/posts/**/*.md")?
        .offset("content")
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

                let buffer = render(
                    ctx,
                    templates,
                    &document.matter,
                    parsed,
                    ctx.env.data.repo.files.get(document.meta.path.as_str()),
                    bibtex.map(|(_, library)| library.path.as_path()),
                    &document.matter.tags,
                    styles,
                    &js,
                )?;

                pages.push(
                    document
                        .output()
                        .strip_prefix("content")?
                        .html()
                        .content(buffer),
                );
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

pub fn render(
    ctx: &Context,
    templates: &TemplateEnv,
    meta: &Post,
    parsed: Parsed,
    info: Option<&GitHistory>,
    library_path: Option<&Utf8Path>,
    tags: &[String],
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<String, RuntimeError> {
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
