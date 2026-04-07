use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::error::HauchiwaError;
use hauchiwa::loader::{Document, Image, Stylesheet, TemplateEnv};
use hauchiwa::prelude::*;
use minijinja::Value;

use crate::md::WikiLinkResolver;
use crate::model::Wiki;
use crate::props::{PropsWiki, PropsWikiBacklink, PropsWikiPdf, PropsWikiTreeNode};
use crate::{Bibtex, Global};

enum RenderedItem<'a> {
    Markdown(&'a Document<Wiki>),
    Typst { title: String },
}

pub fn add_teien(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    images: Many<Image>,
    styles: Many<Stylesheet>,
    bibtex: Many<Bibtex>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let documents = config
        .load_documents::<Wiki>()
        .glob("content/wiki/**/*.md")?
        .offset("content")
        .register();

    let typst = config
        .task()
        .name("wiki:typst:pdf")
        .glob("content/wiki/**/*.typ")?
        .map(|_, store, input| {
            use std::io::Write;
            use std::process::{Command, Stdio};

            let data = input.read().unwrap();

            let mut child = Command::new("typst")
                .arg("c")
                .arg("--format=pdf")
                .arg("-")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            {
                let stdin = child.stdin.as_mut().unwrap();
                stdin.write_all(&data)?;
                stdin.flush()?;
            }

            let output = child.wait_with_output()?;

            if !output.status.success() {
                let stderr = String::from_utf8(output.stderr)?;
                todo!("Typst SSR failed:\n{stderr}")
            }

            let path_pdf = store.save(&output.stdout, "pdf")?;

            Ok((input.path, path_pdf))
        });

    let task = config
        .task()
        .using((templates, documents, images, styles, bibtex, typst))
        .merge(
            |ctx, (templates, documents, images, styles, bibtex, typst)| {
                let styles = &[
                    styles.get("styles/styles.scss")?,
                    styles.get("styles/layouts/page.scss")?,
                ];

                // href -> document
                let doc_map = {
                    let mut doc_map = HashMap::new();

                    for (_, document) in &documents {
                        doc_map.insert(
                            document.meta.href.to_string(),
                            RenderedItem::Markdown(document),
                        );
                    }

                    for (_, typst) in &typst {
                        let href = typst.0.strip_prefix("content/")?.with_extension("");
                        let href = Utf8Path::new("/").join(href);
                        let title = typst.0.file_name().unwrap().to_string();
                        doc_map.insert(href.to_string(), RenderedItem::Typst { title });
                    }

                    doc_map
                };

                // this can track complex relationships between documents
                let mut datalog = crate::datalog::Datalog::new();

                // this can resolve wiki links
                let resolver = WikiLinkResolver::from_assets(&documents);

                let mut typst_items = Vec::new();
                for (_, (path_file, path_pdf)) in &typst {
                    let href = path_file.strip_prefix("content/")?.with_extension("");
                    let href = Utf8Path::new("/").join(href);

                    hauchiwa::tracing::info!("{}", &href);

                    // Datalog: add parent hierarchy
                    {
                        let mut ptr = Utf8Path::new(&href);
                        let mut current_child_str = href.to_string();

                        while let Some(parent) = ptr.parent() {
                            let parent_str = parent.as_str();
                            if parent_str.is_empty() {
                                break;
                            }

                            let parent_normalized = if parent_str == "/" {
                                "/".to_string()
                            } else if !parent_str.ends_with('/') {
                                format!("{}/", parent_str)
                            } else {
                                parent_str.to_string()
                            };

                            datalog.add_parent(&parent_normalized, &current_child_str);
                            current_child_str = parent_normalized;
                            ptr = parent;

                            if ptr == "/" {
                                break;
                            }
                        }
                    }

                    typst_items.push((href.strip_prefix("/")?.to_owned(), path_pdf));
                }

                // pass 1: parse markdown
                let parsed = {
                    let mut parsed = Vec::new();

                    for (_, document) in documents {
                        let library = bibtex.glob(&document.meta.assets("*.bib"))?.next();

                        let markdown = crate::md::parse(
                            &document.text,
                            &document.meta,
                            Some(&resolver),
                            Some(&images),
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
                            hauchiwa::tracing::info!("{}", &href);
                            let mut ptr = Utf8Path::new(&href);
                            let mut current_child_str = href.clone();

                            while let Some(parent) = ptr.parent() {
                                let parent_str = parent.as_str();
                                if parent_str.is_empty() {
                                    break;
                                }

                                let parent_normalized = if parent_str == "/" {
                                    "/".to_string()
                                } else if !parent_str.ends_with('/') {
                                    format!("{}/", parent_str)
                                } else {
                                    parent_str.to_string()
                                };

                                datalog.add_parent(&parent_normalized, &current_child_str);
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
                        let backlinks = solution.get_backlinks(href).map(|hrefs| {
                            hrefs
                                .iter()
                                .filter_map(|h| doc_map.get(*h))
                                .filter_map(|item| match item {
                                    RenderedItem::Markdown(doc) => Some(PropsWikiBacklink {
                                        href: doc.meta.href.clone(),
                                        title: doc.matter.title.clone(),
                                    }),
                                    RenderedItem::Typst { .. } => None,
                                })
                                .collect::<Vec<_>>()
                        });

                        let bibliography = markdown.bibliography.as_ref().map(|bib| {
                            bib.iter()
                                .map(|item| Value::from_safe_string(item.clone()))
                                .collect()
                        });

                        let props = PropsWiki {
                            head: super::make_props_head(
                                ctx,
                                document.matter.title.clone(),
                                styles,
                                &[],
                            )?,
                            navbar: super::make_props_navbar(),
                            footer: super::make_props_footer(ctx),
                            title: document.matter.title.clone(),
                            tree: build_tree_nodes(href, "/", &doc_map, &solution),
                            content: Value::from_safe_string(markdown.html.clone()),
                            bibliography,
                            backlinks,
                        };

                        let tmpl = templates.get_template("wiki.jinja")?;
                        let page = tmpl.render(&props)?;

                        pages.push(
                            document
                                .output()
                                .strip_prefix("content")?
                                .html()
                                .content(page),
                        );
                    }

                    for (href, path_pdf) in &typst_items {
                        let href_full = Utf8Path::new("/").join(href).to_string();

                        let props = PropsWikiPdf {
                            head: super::make_props_head(ctx, "".to_string(), styles, &[])?,
                            navbar: super::make_props_navbar(),
                            footer: super::make_props_footer(ctx),
                            tree: build_tree_nodes(&href_full, "/", &doc_map, &solution),
                            pdf_path: path_pdf.to_string(),
                        };

                        let tmpl = templates.get_template("wiki_pdf.jinja")?;
                        let html = tmpl.render(&props)?;

                        pages.push(Output::html(href, html));
                    }

                    pages
                };

                Ok(pages)
            },
        );

    Ok(task)
}

fn build_tree_nodes(
    active_href: &str,
    parent_href: &str,
    resolved: &HashMap<String, RenderedItem>,
    solution: &crate::datalog::Solution,
) -> Vec<PropsWikiTreeNode> {
    let children = match solution.get_children(parent_href) {
        Some(mut kids) => {
            kids.sort();
            kids
        }
        None => return vec![],
    };

    children
        .into_iter()
        .map(|child_href| {
            let (name, is_link) = if let Some(doc) = resolved.get(child_href) {
                let name = match doc {
                    RenderedItem::Markdown(doc) => doc.matter.title.clone(),
                    RenderedItem::Typst { title } => title.clone(),
                };
                (name, true)
            } else {
                let name = child_href
                    .trim_end_matches('/')
                    .split('/')
                    .next_back()
                    .unwrap_or(child_href)
                    .to_string();
                (name, false)
            };

            let is_active_path = active_href.starts_with(child_href);
            let is_current = active_href == child_href;

            let children = if is_active_path || !is_link {
                build_tree_nodes(active_href, child_href, resolved, solution)
            } else {
                vec![]
            };

            PropsWikiTreeNode {
                href: child_href.to_string(),
                name,
                is_link,
                is_current,
                children,
            }
        })
        .collect()
}
