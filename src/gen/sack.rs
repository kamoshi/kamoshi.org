use std::collections::HashMap;

use camino::Utf8PathBuf;
use hayagriva::Library;

use crate::html::{Link, LinkDate, Linkable};

use super::{load::{Output, OutputKind}, AssetKind};


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


/// This struct allows for querying the website hierarchy. Separate instance of this struct is
/// passed to each closure contained by some rendered assets.
pub struct Sack<'a> {
    /// Literally everything
    hole: &'a [Output],
    /// Current path for page
    path: &'a Utf8PathBuf,
}

impl<'a> Sack<'a> {
    pub fn new(hole: &'a [Output], path: &'a Utf8PathBuf) -> Self {
        Self { hole, path }
    }

    pub fn get_links(&self, path: &str) -> Vec<LinkDate> {
        let pattern = glob::Pattern::new(path).expect("Bad glob pattern");
        self.hole.iter()
            .filter(|item| pattern.matches_path(item.path.as_ref()))
            .filter_map(|item| match &item.link {
                Some(Linkable::Date(link)) => Some(link.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn get_tree(&self, path: &str) -> TreePage {
        let glob = glob::Pattern::new(path).expect("Bad glob pattern");
        let list = self.hole.iter()
            .filter(|item| glob.matches_path(item.path.as_ref()))
            .filter_map(|item| match &item.link {
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
        let glob = glob::Pattern::new(&glob).expect("Bad glob pattern");
        let opts = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: false,
        };

        self.hole.iter()
            .filter(|item| glob.matches_path_with(item.path.as_ref(), opts))
            .filter_map(|asset| match asset.kind {
                OutputKind::Real(ref real) => Some(real),
                _ => None,
            })
            .find_map(|asset| match asset.kind {
                AssetKind::Bibtex(ref lib) => Some(lib),
                _ => None,
            })
    }
}
