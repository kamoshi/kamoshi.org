use camino::Utf8Path;
use hauchiwa::error::RuntimeError;
use hauchiwa::gitmap::GitInfo;
use hauchiwa::loader::{self, CSS, Content, JS, Registry, glob_content};
use hauchiwa::page::{Page, absolutize, normalize_prefixed};
use hauchiwa::task::Handle;
use hauchiwa::{SiteConfig, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Post;
use crate::{Bibtex, CONTENT, Context, Global, Link, LinkDate};

use super::{make_page, render_bibliography, to_list};

pub fn build_posts(
    config: &mut SiteConfig<Global>,
    styles: Handle<Registry<CSS>>,
    scripts: Handle<Registry<JS>>,
) -> Handle<Vec<Page>> {
    let posts = glob_content::<_, Post>(config, "content/posts/**/*.md");

    task!(config, |ctx, posts, styles, scripts| {
        let mut pages = vec![];

        let posts = posts
            .values()
            .filter(|item| !item.metadata.draft)
            .collect::<Vec<_>>();

        // render the posts
        for item in &posts {
            // let pattern = format!("{}/*.bib", item.file.area);
            // let bibtex = ctx.glob::<Bibtex>(&pattern)?.into_iter().next();

            let styles = &[
                styles.get("styles/styles.scss").unwrap(),
                styles.get("styles/layouts/page.scss").unwrap(),
            ];

            let mut js = vec![scripts.get("scripts/outline/main.ts").unwrap()];

            if let Some(entries) = &item.metadata.scripts {
                for entry in entries {
                    let key = format!("scripts/{}", entry);
                    js.push(scripts.get(key).unwrap());
                }
            }

            let article = crate::markdown::parse(&ctx, &item.content, "".into(), None).unwrap();

            let buffer = render(
                &ctx,
                &item.metadata,
                article,
                None,
                None,
                &item.metadata.tags,
                styles,
                &js,
            )
            .unwrap()
            .render();

            pages.push(Page::html(
                item.path.strip_prefix("content/").unwrap(),
                buffer,
            ));
        }

        {
            let styles = &[
                styles.get("styles/styles.scss").unwrap(),
                styles.get("styles/layouts/list.scss").unwrap(),
            ];

            let html = to_list(
                &ctx,
                posts
                    .iter()
                    .map(|item| LinkDate {
                        link: Link {
                            path: absolutize("content", &item.path),
                            name: item.metadata.title.clone(),
                            desc: item.metadata.desc.clone(),
                        },
                        date: item.metadata.date.clone(),
                    })
                    .collect(),
                "Posts".into(),
                "/posts/rss.xml",
                styles,
            )
            .unwrap()
            .render();

            pages.push(Page::html("posts", html));
        }

        {
            // let feed =
            //     crate::rss::generate_feed::<Content<Post>>(sack, "posts", "Kamoshi.org Posts")?;
            // Ok(vec![feed])
        }

        pages
    })
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    meta: &'ctx Post,
    article: Article,
    info: Option<&'ctx GitInfo>,
    library_path: Option<&'ctx Utf8Path>,
    tags: &'ctx [String],
    styles: &'ctx [&CSS],
    scripts: &'ctx [&JS],
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
    info: Option<&GitInfo>,
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
                        a href=(format!("{}/commit/{}", &ctx.data.link, &info.abbreviated_hash)) {
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
