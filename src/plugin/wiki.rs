use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Assets, Document, Image, Script, Stylesheet};
use hauchiwa::page::{Output, absolutize, normalize_prefixed};
use hauchiwa::task::Handle;
use hauchiwa::{Blueprint, task};
use hypertext::{Raw, prelude::*};

use crate::markdown::Article;
use crate::model::Wiki;
use crate::{Context, Global, Link};

use super::{make_page, render_bibliography};

pub fn build_wiki(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let wiki = config.load_documents::<Wiki>("content/wiki/**/*.md")?;

    Ok(task!(config, |ctx, wiki, images, styles| {
        let mut pages = vec![];

        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/page.scss")?,
        ];

        for doc in wiki.values() {
            // let pattern = format!("{}/*", item.file.area);
            // let bibtex = ctx.glob::<Bibtex>(&pattern)?.into_iter().next();

            // let mut js = vec![];

            // for path in &item.metadata {
            //     js.push(path.to_string());
            // }

            let article = crate::markdown::parse(&doc.body, &doc.path, None, Some(images))?;

            let buffer = render(
                ctx,
                &doc.metadata,
                &article,
                "".into(),
                None,
                wiki,
                styles,
                &[],
            )?
            .render();

            pages.push(Output::html(
                normalize_prefixed("content", &doc.path),
                buffer,
            ));
        }

        Ok(pages)
    }))
}

pub fn render<'ctx>(
    ctx: &'ctx Context,
    meta: &'ctx Wiki,
    article: &'ctx Article,
    slug: &'ctx Utf8Path,
    library_path: Option<&'ctx Utf8Path>,
    wiki: &'ctx Assets<Document<Wiki>>,
    styles: &'ctx [&Stylesheet],
    scripts: &'ctx [&Script],
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud!(
        main .wiki-main {
            // Outline
            aside .outline {
                section {
                    div {
                        (show_page_tree(slug, wiki))
                    }
                }
            }
            // Article
            (render_article(meta, article, library_path))
        }
    );

    make_page(ctx, main, meta.title.to_owned(), styles, scripts)
}

fn render_article(
    meta: &Wiki,
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

#[derive(Debug)]
pub struct TreePage {
    pub link: Option<Link>,
    pub subs: HashMap<String, TreePage>,
}

impl TreePage {
    fn new() -> Self {
        TreePage {
            link: None,
            subs: HashMap::new(),
        }
    }

    fn add_link(&mut self, link: &Link) {
        let mut ptr = self;
        for part in link.path.iter().skip(1) {
            ptr = ptr.subs.entry(part.to_string()).or_insert(TreePage::new());
        }
        ptr.link = Some(link.clone());
    }

    fn from_iter(iter: impl Iterator<Item = Link>) -> Self {
        let mut tree = Self::new();
        for link in iter {
            tree.add_link(&link);
        }

        tree
    }
}

/// Render the page tree
pub(crate) fn show_page_tree<'ctx>(
    ctx: &'ctx Utf8Path,
    wiki: &'ctx Assets<Document<Wiki>>,
) -> impl Renderable + use<'ctx> {
    let tree = wiki.values().map(|item| Link {
        path: absolutize("content", &item.path),
        name: item.metadata.title.clone(),
        desc: None,
    });

    let tree = TreePage::from_iter(tree);
    let parts: Vec<_> = ctx.iter().collect();

    maud!(
        h2 .link-tree__heading {
          // {pages.chain(x => x.prefix)
          //   .map(pathify)
          //   .mapOrDefault(href =>
          //     <a class="link-tree__heading-text" href={href}>{heading}</a>,
          //     <span class="link-tree__heading-text">{heading}</span>
          // )}
        }
        nav .link-tree__nav {
            (show_page_tree_level(&tree, &parts))
        }
    )
}

fn show_page_tree_level<'ctx>(
    tree: &'ctx TreePage,
    parts: &'ctx [&str],
) -> impl Renderable + use<'ctx> {
    let subs = {
        let mut subs: Vec<_> = tree.subs.iter().collect();
        subs.sort_by(|a, b| a.0.cmp(b.0));
        subs
    };

    maud!(
        ul .link-tree__nav-list {
            @for (key, next) in &subs {
                li .link-tree__nav-list-item {
                    span .link-tree__nav-list-text {
                        @if let Some(ref link) = next.link {
                            a .link-tree__nav-list-text.link href=(link.path.as_str()) {
                                (&link.name)
                            }
                        } @else {
                            span .link-tree__nav-list-text {
                                (key)
                            }
                        }
                    }
                    @if let Some(part) = parts.first() {
                        @if key == part && !next.subs.is_empty()  {
                            (show_page_tree_level(next, &parts[1..]))
                        }
                    }
                }
            }
        }
    )
}
