use camino::{Utf8Path, Utf8PathBuf};
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::git::GitHistory;
use hauchiwa::loader::{Assets, Document, Image, Script, Stylesheet};
use hauchiwa::page::to_slug;
use hauchiwa::{Blueprint, Handle, Output, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Post;
use crate::{Bibtex, Context, Global, Link, LinkDate};

use super::{make_page, render_bibliography, to_list};

pub fn build_posts(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
    scripts: Handle<Assets<Script>>,
    bibtex: Handle<Assets<Bibtex>>,
) -> Result<(Handle<Assets<Document<Post>>>, Handle<Vec<Output>>), HauchiwaError> {
    let docs = config.load_documents::<Post>("content/posts/**/*.md")?;

    let pages = task!(config, |ctx, docs, images, styles, scripts, bibtex| {
        let mut pages = vec![];

        let documents = docs
            .values()
            .filter(|item| !item.metadata.draft)
            .collect::<Vec<_>>();

        // render the posts
        for document in &documents {
            let pattern = format!("{}/*.bib", to_slug(&document.path));
            let bibtex = bibtex.glob(&pattern)?.into_iter().next();

            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/page.scss")?,
            ];

            let mut js = vec![scripts.get("scripts/outline/main.ts")?];

            if let Some(entries) = &document.metadata.scripts {
                for entry in entries {
                    let key = format!("scripts/{}", entry);
                    js.push(scripts.get(key)?);
                }
            }

            let article = crate::markdown::parse(
                &document.body,
                &document.path,
                bibtex.map(|(_, library)| &library.data),
                Some(images),
            )?;

            let buffer = render(
                ctx,
                &document.metadata,
                article,
                ctx.env.data.repo.files.get(document.path.as_str()),
                bibtex.map(|(_, library)| library.path.as_path()),
                &document.metadata.tags,
                styles,
                &js,
            )?
            .render()
            .into_inner();

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
                documents
                    .iter()
                    .map(|item| LinkDate {
                        link: Link {
                            path: Utf8PathBuf::from(item.href("content")),
                            name: item.metadata.title.clone(),
                            desc: item.metadata.desc.clone(),
                        },
                        date: item.metadata.date,
                    })
                    .collect(),
                "Posts".into(),
                "/posts/rss.xml",
                styles,
            )?
            .render()
            .into_inner();

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

pub fn render<'ctx>(
    ctx: &'ctx Context,
    meta: &'ctx Post,
    article: Article,
    info: Option<&'ctx GitHistory>,
    library_path: Option<&'ctx Utf8Path>,
    tags: &'ctx [String],
    styles: &'ctx [&Stylesheet],
    scripts: &'ctx [&Script],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud!(
        main {
            // Outline (left)
            (&article.outline)
            // Article (center)
            (render_article(meta, &article, library_path))
            // Metadata (right)
            (render_metadata(ctx, meta, info, tags))
        }
    );

    make_page(ctx, main, meta.title.clone(), styles, scripts)
}

fn render_article(
    meta: &Post,
    article: &Article,
    library_path: Option<&Utf8Path>,
) -> impl Renderable {
    maud!(
        article .article {
            section .paper {
                header {
                    h1 #top {
                        (&meta.title)
                    }
                }
                section .wiki-article__markdown.markdown {
                    (Raw::dangerously_create(&article.text))
                }
            }

            @if let Some(bib) = &article.bibliography.0 {
                (render_bibliography(bib, library_path))
            }
        }
    )
}

pub fn render_metadata(
    ctx: &Context,
    meta: &Post,
    info: Option<&GitHistory>,
    tags: &[String],
) -> impl Renderable {
    maud!(
        aside .tiles {
            section .metadata {
                h2 {
                    "Metadata"
                }
                div {
                    img src="/static/svg/lucide/file-plus-2.svg" title="Added";
                    time datetime=(meta.date.format("%Y-%m-%d").to_string()) {
                        (meta.date.format("%Y, %B %d").to_string())
                    }
                }
                @if let Some(info) = info {
                    @let info = info[0].as_ref();
                    div {
                        img src="/static/svg/lucide/file-clock.svg" title="Updated";
                        time datetime=(info.commit_date.format("%Y-%m-%d").to_string()) {
                            (info.commit_date.format("%Y, %B %d").to_string())
                        }
                    }
                    div {
                        img src="/static/svg/lucide/git-graph.svg" title="Link to commit";
                        a href=(format!("{}/commit/{}", &ctx.env.data.link, &info.abbreviated_hash)) {
                            (&info.abbreviated_hash)
                        }
                    }
                }
                @if !tags.is_empty() {
                    div .tags {
                        img src="/static/svg/lucide/tag.svg" title="Tags";
                        ul {
                            @for tag in tags {
                                li {
                                    a href=(format!("/tags/{tag}")) {
                                        (tag)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}
