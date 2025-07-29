use camino::Utf8Path;
use hauchiwa::loader::{self, Content, Script, yaml};
use hauchiwa::{Page, Plugin, RuntimeError};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Post;
use crate::{Bibtex, CONTENT, Context, Global, LinkDate, Outline};

use super::{make_page, render_bibliography, to_list};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "posts/**/*.md", yaml::<Post>),
        ])
        .add_task("posts", |ctx| {
            let mut pages = vec![];

            for item in ctx
                .glob_with_file::<Content<Post>>("posts/**/*")?
                .into_iter()
                .filter(|item| !item.data.meta.draft)
            {
                let pattern = format!("{}/*.bib", item.file.area);
                let bibtex = ctx.glob::<Bibtex>(&pattern)?.into_iter().next();

                let article = crate::markdown::parse(
                    &ctx,
                    &item.data.text,
                    &item.file.area,
                    bibtex.map(|x| &x.data),
                )?;

                let buffer = render(
                    &ctx,
                    &item.data.meta,
                    article,
                    item.file.info.as_ref(),
                    bibtex.map(|x| x.path.as_ref()),
                    &item.data.meta.tags,
                )?
                .render();

                pages.push(Page::text(item.file.area.join("index.html"), buffer))
            }
            Ok(pages)
        })
        .add_task("posts_list", |ctx| {
            Ok(vec![Page::text(
                "posts/index.html",
                to_list(
                    &ctx,
                    ctx.glob_with_file::<Content<Post>>("posts/**/*")?
                        .iter()
                        .filter(|item| !item.data.meta.draft)
                        .map(LinkDate::from)
                        .collect(),
                    "Posts".into(),
                    "/posts/rss.xml",
                )?
                .render(),
            )])
        })
        .add_task("posts_feed", |sack| {
            let feed =
                crate::rss::generate_feed::<Content<Post>>(sack, "posts", "Kamoshi.org Posts")?;
            Ok(vec![feed])
        });
});

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles.scss", "layouts/page.scss"];

pub fn render<'ctx>(
    ctx: &'ctx Context,
    meta: &'ctx Post,
    article: Article,
    info: Option<&'ctx hauchiwa::GitInfo>,
    library_path: Option<&'ctx Utf8Path>,
    tags: &'ctx [String],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let mut scripts = vec![];

    for path in &article.scripts {
        scripts.push(path.to_string());
    }

    for script in meta.scripts.iter().flatten() {
        let script = ctx.get::<Script>(script)?;
        scripts.push(script.path.to_string());
    }

    let main = maud!(
        main {
            // Outline (left)
            (render_outline(&article.outline))
            // Article (center)
            (render_article(meta, &article, library_path))
            // Metadata (right)
            (render_metadata(ctx, meta, info, tags))
        }
    );

    make_page(ctx, main, meta.title.clone(), STYLES, scripts.into())
}

pub fn render_outline(outline: &Outline) -> impl Renderable {
    maud!(
        aside .outline {
            section {
                h2 {
                    a href="#top" { "Outline" }
                }
                nav #table-of-contents {
                    ul {
                        @for (title, id) in &outline.0 {
                            li {
                                a href=(format!("#{id}")) {
                                    (title)
                                }
                            }
                        }
                    }
                }
            }
        }
    )
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
                    (Raw(&article.text))
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
    info: Option<&hauchiwa::GitInfo>,
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
                    div {
                        img src="/static/svg/lucide/file-clock.svg" title="Updated";
                        time datetime=(info.commit_date.format("%Y-%m-%d").to_string()) {
                            (info.commit_date.format("%Y, %B %d").to_string())
                        }
                    }
                    div {
                        img src="/static/svg/lucide/git-graph.svg" title="Link to commit";
                        a href=(format!("{}/commit/{}", &ctx.get_globals().data.link, &info.abbreviated_hash)) {
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
