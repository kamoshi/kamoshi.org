use std::collections::HashMap;

use camino::Utf8PathBuf;
use hayagriva::Library;

use crate::html::{Link, LinkDate, Linkable};

use super::{Asset, AssetKind};


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
pub struct Sack<'a> {
    assets: &'a [&'a Asset],
    path: &'a Utf8PathBuf,
}

impl<'a> Sack<'a> {
    pub fn new(assets: &'a [&'a Asset], path: &'a Utf8PathBuf) -> Self {
        Self { assets, path }
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

    pub fn get_library(&self) -> Option<&Library> {
        let glob = format!("{}/*.bib", self.path.parent()?);
        let glob = glob::Pattern::new(&glob).unwrap();
        let opts = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: false,
        };

        self.assets.iter()
            .filter(|asset| glob.matches_path_with(asset.out.as_ref(), opts))
            .find_map(|asset| match asset.kind {
                AssetKind::Bib(ref lib) => Some(lib),
                _ => None
            })
    }
}
