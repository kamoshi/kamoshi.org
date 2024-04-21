use std::collections::HashMap;

use hypertext::{html_elements, maud_move, GlobalAttributes, Renderable};

use crate::md::Wiki;
use crate::html::page;
use crate::text::md::Outline;
use crate::Sack;

use super::Link;


#[derive(Debug)]
struct TreeNode {
    pub name: String,
    pub children: HashMap<String, TreeNode>,
}

impl TreeNode {
    fn new(name: &str) -> Self {
        TreeNode {
            name: name.to_string(),
            children: HashMap::new(),
        }
    }

    fn add_link(&mut self, link: &Link) {
        let mut current_node = self;
        for component in link.path.split('/').filter(|s| !s.is_empty()) {
            current_node = current_node.children.entry(component.to_string())
                .or_insert(TreeNode::new(component));
        }
    }
}

fn tree(sack: &Sack) -> impl Renderable {
    let mut tree = TreeNode::new("wiki");
    for link in sack.get_links_2("wiki/**/*.html") {
        tree.add_link(&link);
    };

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
          // {pages.map(pages => <PagesList {...pages} />).extract()}
            (level(&tree))
        }
    )
}

fn level(tree: &TreeNode) -> impl Renderable + '_ {
    for (key, next) in tree.children.iter() {
        println!("{key}");
        level(next);
    };
    maud_move!(
        ul .link-tree__nav-list {
            @for (key, next) in tree.children.iter() {
                li .link-tree__nav-list-item {
                    span .link-tree__nav-list-text { (key) }
                    @if next.children.len() > 0 {
                        (level(next))
                    }
                }
            }
        }
    )
}

// {tree.children
//   .map(m => Object.values(m))
//   .filter(xs => xs.length > 0)
//   .map(pages =>
//     <ul class="link-tree__nav-list">
//       {pages
//         .sort(compare)
//         .map(page => ({...page, current: checkCurrent(page.slug) }))
//         .map(page =>
//           <li class="link-tree__nav-list-item">
//             {page.slug
//               .chain(slug => prefix.map(prefix => pathify(prefix, slug)))
//               .map(href => (page.current)
//                 ? <button id="current-page-button" class="link-tree__nav-list-text current">{page.title}</button>
//                 : <a class="link-tree__nav-list-text link" href={href}>{page.title}</a>
//               )
//               .orDefault(<span class="link-tree__nav-list-text">{page.title}</span>)}
//             <Astro.self tree={page} slug={slug} prefix={prefix} />
//           </li>
//       )}
//     </ul>
// ).extract()}


pub fn wiki<'data, 'html, 'sack, T>(
    fm: &'data Wiki,
    content: T,
    outline: Outline,
    sack: &'sack Sack,
) -> impl Renderable + 'html
    where
        'sack: 'html,
        'data: 'html,
        T: Renderable + 'data
{
    let main = maud_move!(
        main .wiki-main {

            // Slide in/out for mobile
            input #wiki-aside-shown type="checkbox" hidden;

            aside .wiki-aside {
                // Slide button
                label .wiki-aside__slider for="wiki-aside-shown" {
                    img .wiki-icon src="/static/svg/double-arrow.svg" width="24" height="24";
                }
                // Navigation tree
                section .link-tree {
                    div {
                        (tree(sack))
                    }
                }
            }

            article .wiki-article /*class:list={classlist)*/ {
                header class="markdown" {
                    h1 #top { (fm.title.clone()) }
                }
                section .wiki-article__markdown.markdown {
                    (content)
                }
            }
        }
    );

    page(&fm.title, main)
}
