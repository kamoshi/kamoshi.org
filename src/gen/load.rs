use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};
use glob::glob;
use hayagriva::Library;

use crate::html::Linkable;

use super::Sack;


/// Marks whether the item should be treated as a content page, converted into a standalone HTML
/// page, or as a bundled asset.
#[derive(Debug)]
pub enum FileItemKind {
    /// Convert to `index.html`
    Index,
    /// Convert to a bundled asset
    Bundle,
}

/// Metadata for a single item consumed by SSG.
#[derive(Debug)]
pub struct FileItem {
    /// Kind of an item
    pub kind: FileItemKind,
    /// Original source file location
    pub path: Utf8PathBuf,
}

/// Marks how the asset should be processed by the SSG
pub enum AssetKind {
    /// Data renderable to HTML
    Html(Box<dyn Fn(&Sack) -> String>),
    /// Bibliographical data
    Bibtex(Library),
    /// Images
    Image,
}

/// Asset renderable by the SSG
pub struct Asset {
    /// Kind of a processed asset
    pub kind: AssetKind,
    /// File metadata
    pub meta: FileItem,
}

/// Dynamically generated asset not related to any disk file.
pub struct Dynamic(pub Box<dyn Fn(&Sack) -> String>);

impl Dynamic {
    pub fn new(call: impl Fn(&Sack) -> String + 'static) -> Self {
        Self(Box::new(call))
    }
}

pub enum OutputKind {
    Real(Asset),
    Fake(Dynamic),
}

impl From<Asset> for OutputKind {
    fn from(value: Asset) -> Self {
        OutputKind::Real(value)
    }
}

impl From<Dynamic> for OutputKind {
    fn from(value: Dynamic) -> Self {
        OutputKind::Fake(value)
    }
}

/// Renderable output
pub struct Output {
    pub kind: OutputKind,
    pub path: Utf8PathBuf,
    /// Optional link to outputted page.
    pub link: Option<Linkable>,
}

/// Variants used for filtering static assets.
pub enum PipelineItem {
    /// Unclaimed file, unrecognized file extensions.
    Skip(FileItem),
    /// Data ready to be processed by SSG.
    Take(Output),
}

impl From<FileItem> for PipelineItem {
    fn from(value: FileItem) -> Self {
        Self::Skip(value)
    }
}

impl From<Output> for PipelineItem {
    fn from(value: Output) -> Self {
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


fn to_source(path: Utf8PathBuf, exts: &HashSet<&'static str>) -> FileItem {
    let hit = path.extension().map_or(false, |ext| exts.contains(ext));

    let kind = match hit {
        true  => FileItemKind::Index,
        false => FileItemKind::Bundle,
    };

    FileItem {
        kind,
        path,
    }
}


pub fn render_all(items: &[Output]) {
    for item in items {
        let file = match &item.kind {
            OutputKind::Real(a) => Some(&a.meta.path),
            OutputKind::Fake(_) => None,
        };
        render(item, &Sack::new(items, &item.path, file));
    }
}

fn render(item: &Output, sack: &Sack) {
    let o = Utf8Path::new("dist").join(&item.path);
    fs::create_dir_all(o.parent().unwrap()).unwrap();

    match item.kind {
        OutputKind::Real(ref real) => {
            let i = &real.meta.path;

            match &real.kind {
                AssetKind::Html(closure) => {
                    let mut file = File::create(&o).unwrap();
                    file.write_all(closure(sack).as_bytes()).unwrap();
                    println!("HTML: {} -> {}", i, o);
                },
                AssetKind::Bibtex(_) => { },
                AssetKind::Image => {
                    fs::create_dir_all(o.parent().unwrap()).unwrap();
                    fs::copy(i, &o).unwrap();
                    println!("Image: {} -> {}", i, o);
                },
            };
        },
        OutputKind::Fake(Dynamic(ref closure)) => {
            let mut file = File::create(&o).unwrap();
            file.write_all(closure(sack).as_bytes()).unwrap();
            println!("Virtual: -> {}", o);
        },
    }
}
