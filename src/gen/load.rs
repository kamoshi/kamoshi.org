use std::collections::HashSet ;

use camino::Utf8PathBuf;
use glob::glob;
use hayagriva::Library;

use crate::html::Linkable;

use super::Sack;


/// Whether the item should be treated as a content page, converted into a standalone HTML page, or
/// as a bundled asset.
#[derive(Debug)]
pub enum StaticItemKind {
    /// Convert to `index.html`
    Index,
    /// Convert to a bundled asset
    Bundle,
}

/// Metadata for a single item consumed by SSG.
#[derive(Debug)]
pub struct StaticItem {
    /// Kind of an item
    pub kind: StaticItemKind,
    /// Original extension for the source file
    pub ext: String,
    pub dir: Utf8PathBuf,
    pub src: Utf8PathBuf,
}

pub enum AssetKind {
    Html(Box<dyn Fn(&Sack) -> String>),
    Image,
    Other,
    Bib(Library),
}

pub struct Asset {
    pub kind: AssetKind,
    pub out: Utf8PathBuf,
    pub meta: StaticItem,
    pub link: Option<Linkable>,
}

pub enum PipelineItem {
    Skip(StaticItem),
    Take(Asset),
}

impl From<StaticItem> for PipelineItem {
    fn from(value: StaticItem) -> Self {
        Self::Skip(value)
    }
}

impl From<Asset> for PipelineItem {
    fn from(value: Asset) -> Self {
        Self::Take(value)
    }
}


pub fn gather(pattern: &str, exts: &HashSet<&'static str>) -> Vec<PipelineItem> {
    glob(pattern)
        .expect("Invalid glob pattern")
        .filter_map(|path| {
            let path = path.unwrap();
            let path = Utf8PathBuf::from_path_buf(path).expect("Filename is not valid UTF8");

            match path.is_dir() {
                true  => None,
                false => Some(to_source(path, exts))
            }
        })
        .map(Into::into)
        .collect()
}


fn to_source(path: Utf8PathBuf, exts: &HashSet<&'static str>) -> StaticItem {
    let dir = path.parent().unwrap();
    let ext = path.extension().unwrap();

    if !exts.contains(ext) {
        return StaticItem {
            kind: StaticItemKind::Bundle,
            ext: ext.to_owned(),
            dir: dir.to_owned(),
            src: path,
        };
    }

    let dirs = match path.file_stem().unwrap() {
        "index" => dir.to_owned(),
        name    => dir.join(name),
    };

    StaticItem {
        kind: StaticItemKind::Index,
        ext: ext.to_owned(),
        dir: dirs,
        src: path,
    }
}
