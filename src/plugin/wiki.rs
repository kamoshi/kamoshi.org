use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::error::HauchiwaError;
use hauchiwa::loader::{Assets, Document, Image, Stylesheet};
use hauchiwa::page::{absolutize, normalize_prefixed};
use hauchiwa::{Blueprint, Handle, Output, task};
use hypertext::{Raw, maud_borrow, prelude::*};

use crate::md::WikiLinkResolver;
use crate::model::Wiki;
use crate::plugin::datalog::Datalog;
use crate::{Global, Link};

use super::make_page;

pub fn build(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let input = config.load_documents::<Wiki>("content/wiki/**/*.md")?;

    Ok(task!(config, |ctx, input, images, styles| {
        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/page.scss")?,
        ];

        // documents ordered sequentially, index can be used as datalog id
        let documents = input
            .values()
            .map(|document| (document.href("content/"), document))
            .collect::<Vec<_>>();

        // this can track complex relationships between documents
        let mut datalog = Datalog::new();

        // this can resolve wiki links
        let resolver = WikiLinkResolver::from_assets::<Wiki>("content/", input);

        // pass 1: parse markdown
        let parsed = {
            let mut parsed = Vec::new();

            for (id, (href, doc)) in documents.iter().enumerate() {
                let (html, refs) = crate::md::parse_markdown(&doc.body, &resolver)?;

                for target in &refs {
                    if let Some(target) = documents.iter().position(|(href, _)| href == target) {
                        datalog.add_link(id, target);
                    }
                }

                parsed.push((id, doc, html, href));
            }

            parsed
        };

        // here we can solve the datalog rules
        let solution = datalog.solve();

        // this tracks the parent child relationships between documents, for now
        let tree = TreePage::from_iter(input.values().map(|item| Link {
            path: absolutize("content", &item.path),
            name: item.metadata.title.clone(),
            desc: None,
        }));

        // pass 2: render html
        let pages = {
            let mut pages = vec![];

            for (id, document, html, href) in &parsed {
                let path_parts = Utf8Path::new(href)
                    .strip_prefix("/")
                    .unwrap()
                    .iter()
                    .collect::<Vec<_>>();

                let backrefs = solution.get_backlinks(*id).map(|slice| {
                    slice
                        .iter()
                        .map(|index| &documents[*index])
                        .collect::<Vec<_>>()
                });

                let main = maud_borrow!(
                    main .wiki-main {
                        // Outline
                        aside .outline {
                            section {
                                div {
                                    (show_page_tree(&tree, &path_parts))
                                }
                            }
                        }

                        // Article
                        (render_article(&document.metadata, html, backrefs.as_deref()))
                    }
                );

                let page = make_page(ctx, main, document.metadata.title.to_owned(), styles, &[])?
                    .render()
                    .into_inner();

                pages.push(Output::html(
                    normalize_prefixed("content", &document.path),
                    page,
                ));
            }

            pages
        };

        Ok(pages)
    }))
}

fn render_article(
    meta: &Wiki,
    text: &str,
    backlinks: Option<&[&(String, &Document<Wiki>)]>,
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
                    (Raw::dangerously_create(text))
                }
            }

            // @if let Some(bib) = &article.bibliography.0 {
            //     (render_bibliography(bib, library_path))
            // }

            // Backlinks Footer
            @if let Some(backlinks) = backlinks {
                div {
                    h3 { "Backlinks" }

                    ul .backlinks-list {
                        @for link in backlinks {
                            li {
                                "Is referenced by "
                                a href=(link.0) { (&link.1.metadata.title) }
                            }
                        }
                    }
                }
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
    tree: &'ctx TreePage,
    path: &'ctx [&str],
) -> impl Renderable + use<'ctx> {
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
            (show_page_tree_level(tree, path))
        }
    )
}

fn show_page_tree_level<'ctx>(
    tree: &'ctx TreePage,
    path: &'ctx [&str],
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
                    @if let Some(part) = path.first() {
                        @if key == part && !next.subs.is_empty()  {
                            (show_page_tree_level(next, &path[1..]))
                        }
                    }
                }
            }
        }
    )
}
