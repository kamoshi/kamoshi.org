use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::error::HauchiwaError;
use hauchiwa::loader::{Assets, Document, Image, Stylesheet};
use hauchiwa::{Blueprint, Handle, Output};
use hypertext::{Raw, maud_borrow, prelude::*};

use crate::Global;
use crate::md::WikiLinkResolver;
use crate::model::Wiki;

use super::make_page;

pub fn build(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let documents = config
        .load_documents::<Wiki>()
        .source("content/wiki/**/*.md")
        .offset("content")
        .register()?;

    let task = hauchiwa::task!(config, |ctx, documents, images, styles| {
        let styles = &[
            styles.get("styles/styles.scss")?,
            styles.get("styles/layouts/page.scss")?,
        ];

        // href -> document
        let doc_map = {
            let mut doc_map = HashMap::new();

            for document in documents.values() {
                doc_map.insert(document.href.as_str(), document);
            }

            doc_map
        };

        // this can track complex relationships between documents
        let mut datalog = crate::datalog::Datalog::new();

        // this can resolve wiki links
        let resolver = WikiLinkResolver::from_assets(documents);

        // pass 1: parse markdown
        let parsed = {
            let mut parsed = Vec::new();

            for document in documents.values() {
                let (html, refs) = crate::md::parse_markdown(
                    &document.body,
                    &document.path,
                    &resolver,
                    Some(images),
                )?;

                let href = document.href.clone();

                // Datalog: add wiki links
                for target_href in &refs {
                    if doc_map.contains_key(target_href.as_str()) {
                        datalog.add_link(&href, target_href);
                    }
                }

                // Datalog: add parent hierarchy
                {
                    let mut ptr = Utf8Path::new(&href);

                    // Track the current child (start with the document itself)
                    let mut current_child_str = href.clone();

                    while let Some(parent) = ptr.parent() {
                        let parent_str = parent.as_str();
                        if parent_str.is_empty() {
                            break;
                        }

                        // Normalize parent to ensure trailing slash
                        let parent_normalized = if parent_str == "/" {
                            "/".to_string()
                        } else if !parent_str.ends_with('/') {
                            format!("{}/", parent_str)
                        } else {
                            parent_str.to_string()
                        };

                        // add link Parent -> Child
                        datalog.add_parent(&parent_normalized, &current_child_str);

                        // The parent becomes the child for the next iteration
                        current_child_str = parent_normalized;
                        ptr = parent;

                        if ptr == "/" {
                            break;
                        }
                    }
                }

                parsed.push((document, html, href));
            }

            parsed
        };

        // here we can solve the datalog rules
        let solution = datalog.solve();

        // pass 2: render html
        let pages = {
            let mut pages = vec![];

            for (document, html, href) in &parsed {
                // Get backlinks (list of href strings) and map them to Document objects
                let backrefs = solution.get_backlinks(href).map(|hrefs| {
                    hrefs
                        .iter()
                        .filter_map(|h| doc_map.get(*h))
                        .map(|&doc| (doc.href.as_str(), doc)) // Tuple for the template
                        .collect::<Vec<_>>()
                });

                let main = maud_borrow!(
                    main .wiki-main {
                        // Outline
                        aside .outline {
                            section {
                                div {
                                    (TreeContext::new("/", href, &doc_map, &solution))
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

                pages.push(
                    document
                        .output()
                        .strip_prefix("content")?
                        .html()
                        .content(page),
                );
            }

            pages
        };

        Ok(pages)
    });

    Ok(task)
}

fn render_article(
    meta: &Wiki,
    text: &str,
    backlinks: Option<&[(&str, &Document<Wiki>)]>,
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

            @if let Some(backlinks) = backlinks {
                div .backlinks {
                    h3 { "Backlinks" }

                    ul {
                        @for link in backlinks {
                            li {
                                a href=(link.0) { (&link.1.metadata.title) }
                            }
                        }
                    }
                }
            }
        }
    )
}

// Helper struct to bundle the context needed for rendering
struct TreeContext<'a> {
    root: &'a str,
    href: &'a str,
    solution: &'a crate::datalog::Solution,
    resolved: &'a HashMap<&'a str, &'a Document<Wiki>>,
}

impl<'a> TreeContext<'a> {
    fn new(
        root: &'a str,
        href: &'a str,
        resolved: &'a HashMap<&str, &Document<Wiki>>,
        solution: &'a crate::datalog::Solution,
    ) -> Self {
        Self {
            root,
            href,
            solution,
            resolved,
        }
    }
}

impl hypertext::Renderable for TreeContext<'_> {
    fn render_to(&self, buffer: &mut hypertext::Buffer<hypertext::context::Node>) {
        maud!(
            nav .link-tree__nav {
                (show_tree_recursive(self, self.root))
            }
        )
        .render_to(buffer);
    }
}

fn show_tree_recursive(ctx: &TreeContext<'_>, href: &str) -> impl Renderable {
    let children = ctx.solution.get_children(href).map(|mut kids| {
        kids.sort();
        kids
    });

    maud!(
        @if let Some(children) = &children {
            ul .link-tree__nav-list {
                @for child_href in children {
                    // Determine display name: Title if doc exists, else directory name
                    @let (name, is_link) = if let Some(doc) = ctx.resolved.get(*child_href) {
                        (doc.metadata.title.as_str(), true)
                    } else {
                        // Fallback: extract last folder name from "/wiki/cs/languages/" -> "languages"
                        let name = child_href.trim_end_matches('/').split('/').next_back().unwrap_or(child_href);
                        (name, false)
                    };

                    // Check if this child is part of the active path (to expand it)
                    // e.g. if active is "/wiki/cs/datalog/", then "/wiki/cs/" is active
                    @let is_active_path = ctx.href.starts_with(child_href);
                    @let is_current_page = ctx.href == *child_href;

                    li .link-tree__nav-list-item {
                        span .link-tree__nav-list-text {
                            @if is_link {
                                a .link-tree__nav-list-text.link
                                    .active[is_current_page]
                                    href=(child_href)
                                {
                                    (name)
                                }
                            } @else {
                                // a hole
                                span .link-tree__nav-list-text { (name) }
                            }
                        }

                        // expand children if this node is part of the active
                        // path, or if it's not a link
                        @if is_active_path || !is_link {
                            (show_tree_recursive(ctx, child_href))
                        }
                    }
                }
            }
        }
    )
}
