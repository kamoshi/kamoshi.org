use std::collections::HashMap;

use camino::Utf8Path;
use hauchiwa::loader::{self, Content, yaml};
use hauchiwa::{Page, Plugin, RuntimeError};
use hypertext::{GlobalAttributes, Raw, Renderable, html_elements, maud_move};

use crate::markdown::Article;
use crate::model::Wiki;
use crate::shared::{make_page, render_bibliography};
use crate::{Bibtex, CONTENT, Context, Global, Link};

pub const PLUGIN: Plugin<Global> = Plugin::new(|config| {
    config
        .add_loaders([
            //
            loader::glob_content(CONTENT, "wiki/**/*.md", yaml::<Wiki>),
        ])
        .add_task("wiki", |ctx| {
            let mut pages = vec![];

            for item in ctx.glob_with_file::<Content<Wiki>>("**/*")? {
                let pattern = format!("{}/*", item.file.area);
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
                    &article,
                    &item.file.area,
                    bibtex.map(|x| x.path.as_ref()),
                )?
                .render();

                pages.push(Page::html(&item.file.area, buffer));
            }

            Ok(pages)
        });
});

/// Styles relevant to this fragment
const STYLES: &[&str] = &["styles.scss", "layouts/page.scss"];

pub fn render<'ctx>(
    ctx: &'ctx Context,
    meta: &'ctx Wiki,
    article: &'ctx Article,
    slug: &'ctx Utf8Path,
    library_path: Option<&'ctx Utf8Path>,
) -> Result<impl Renderable + use<'ctx>, RuntimeError> {
    let main = maud_move!(
        main .wiki-main {
            // Outline
            aside .outline {
                section {
                    div {
                        (show_page_tree(slug, ctx))
                    }
                }
            }
            // Article
            (render_article(meta, article, library_path))
        }
    );

    make_page(ctx, main, meta.title.to_owned(), STYLES, Default::default())
}

fn render_article(
    meta: &Wiki,
    article: &Article,
    library_path: Option<&Utf8Path>,
) -> impl Renderable {
    maud_move!(
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
    sack: &'ctx Context,
) -> impl Renderable + use<'ctx> {
    let tree = sack
        .glob_with_file::<Content<Wiki>>("**/*")
        .unwrap()
        .into_iter()
        .map(|item| Link {
            path: Utf8Path::new("/").join(&item.file.area),
            name: item.data.meta.title.clone(),
            desc: None,
        });

    let tree = TreePage::from_iter(tree);
    let parts: Vec<_> = ctx.iter().collect();

    maud_move!(
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

    maud_move!(
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
