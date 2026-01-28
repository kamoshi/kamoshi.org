use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::error::HauchiwaError;
use hauchiwa::loader::{Assets, Document, Image, Stylesheet};
use hauchiwa::{Blueprint, Handle, Output};
use hypertext::{Raw, maud_borrow, prelude::*};

use crate::md::{Parsed, WikiLinkResolver};
use crate::model::Wiki;
use crate::{Bibtex, Global};

use super::make_page;

pub fn add_teien(
    config: &mut Blueprint<Global>,
    images: Handle<Assets<Image>>,
    styles: Handle<Assets<Stylesheet>>,
    bibtex: Handle<Assets<Bibtex>>,
) -> Result<Handle<Vec<Output>>, HauchiwaError> {
    let documents = config
        .load_documents::<Wiki>()
        .source("content/wiki/**/*.md")
        .offset("content")
        .register()?;

    let task = config
        .task()
        .depends_on((documents, images, styles, bibtex))
        .run(|ctx, (documents, images, styles, bibtex)| {
            let styles = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/page.scss")?,
            ];

            // href -> document
            let doc_map = {
                let mut doc_map = HashMap::new();

                for document in documents.values() {
                    doc_map.insert(document.meta.href.as_str(), document);
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
                    let library = bibtex
                        .glob(&document.meta.assets("*.bib"))?
                        .into_iter()
                        .next();

                    let markdown = crate::md::parse(
                        &document.text,
                        &document.meta,
                        Some(&resolver),
                        Some(images),
                        library.map(|library| &library.1.data),
                    )?;

                    let href = document.meta.href.clone();

                    // Datalog: add wiki links
                    for target_href in &markdown.refs {
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

                    parsed.push((document, markdown, href));
                }

                parsed
            };

            // here we can solve the datalog rules
            let solution = datalog.solve();

            // pass 2: render html
            let pages = {
                let mut pages = vec![];

                for (document, markdown, href) in &parsed {
                    // Get backlinks (list of href strings) and map them to Document objects
                    let backrefs = solution.get_backlinks(href).map(|hrefs| {
                        hrefs
                            .iter()
                            .filter_map(|h| doc_map.get(*h))
                            .map(|&doc| (doc.meta.href.as_str(), doc)) // Tuple for the template
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
                            (render_article(&document.matter, markdown, backrefs.as_deref()))
                        }
                    );

                    let page = make_page(ctx, main, document.matter.title.to_owned(), styles, &[])?
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
    markdown: &Parsed,
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
                    (Raw::dangerously_create(&markdown.html))
                }
            }

            @if let Some(bibliography) = &markdown.bibliography {
                (render_bibliography(bibliography))
            }

            @if let Some(backlinks) = backlinks {
                div .backlinks {
                    h3 { "Backlinks" }

                    ul {
                        @for link in backlinks {
                            li {
                                a href=(link.0) { (&link.1.matter.title) }
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
                        (doc.matter.title.as_str(), true)
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

pub fn render_bibliography(bibliography: &[String]) -> impl Renderable {
    maud!(
        section .bibliography {
            header {
                h2 {
                    "Bibliography"
                }
                // @if let Some(path) = library_path {
                //     a.icon-btn href=(path.as_str()) download="bibliography.bib" title="Download BibTeX" {
                //         img src="/static/svg/lucide/file-down.svg" alt="Download";
                //     }
                // }
            }
            ol {
                @for item in bibliography {
                    li {
                        (Raw::dangerously_create(item))
                    }
                }
            }
        }
    )
}
