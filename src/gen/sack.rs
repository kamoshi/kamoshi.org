use std::collections::HashMap;

use crate::html::{Link, LinkDate, Linkable};

use super::Asset;


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
            ptr = ptr.subs
                .entry(part.to_string())
                .or_insert(TreePage::new());
        }
        ptr.link = Some(link.clone());
    }
}


/// This struct allows for querying the website hierarchy.
#[derive(Debug)]
pub struct Sack<'a> {
    assets: &'a [&'a Asset],
}

impl<'a> Sack<'a> {
    pub fn new(assets: &'a [&'a Asset]) -> Self {
        Self { assets }
    }

    pub fn get_links(&self, path: &str) -> Vec<LinkDate> {
        let pattern = glob::Pattern::new(path).unwrap();
        self.assets.iter()
            .filter(|f| pattern.matches_path(f.out.as_ref()))
            .filter_map(|f| match &f.link {
                Some(Linkable::Date(link)) => Some(link.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn get_tree(&self, path: &str) -> TreePage {
        let glob = glob::Pattern::new(path).unwrap();
        let list = self.assets.iter()
            .filter(|f| glob.matches_path(f.out.as_ref()))
            .filter_map(|f| match &f.link {
                Some(Linkable::Link(link)) => Some(link.clone()),
                _ => None,
            });

        let mut tree = TreePage::new();
        for link in list {
            tree.add_link(&link);
        };

        tree
    }
}
