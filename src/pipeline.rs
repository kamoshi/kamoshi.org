//! The purpose of this module is to process the data loaded from content files, which involves
//! loading the data from hard drive, and then processing it further depending on the file type.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};
use glob::glob;
use hayagriva::Library;
use hypertext::Renderable;

use crate::text::md::Outline;
use crate::{Link, LinkDate, Linkable};

/// Represents a piece of content that can be rendered as a page. This trait needs to be
/// implemented for the front matter associated with some web page as that is what ultimately
/// matters when rendering the page. Each front matter *definition* maps to exactly one kind of
/// rendered page on the website.
pub(crate) trait Content {
    /// Parse the document. Pass an optional library for bibliography.
    fn parse(document: &str, library: Option<&Library>) -> (Outline, String, Option<Vec<String>>);

    fn transform<'fm, 'md, 'sack, 'html, T>(
        &'fm self,
        content: T,
        outline: Outline,
        sack: &'sack Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
    where
        'fm: 'html,
        'md: 'html,
        'sack: 'html,
        T: Renderable + 'md;

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable>;
}

/// Marks whether the item should be treated as a content page, converted into a standalone HTML
/// page, or as a bundled asset.
#[derive(Debug)]
pub(crate) enum FileItemKind {
    /// Marks items converted to `index.html`.
    Index,
    /// Marks items from bundle.
    Bundle,
}

/// Metadata for a single item consumed by SSG.
#[derive(Debug)]
pub(crate) struct FileItem {
    /// The kind of an item from disk.
    pub kind: FileItemKind,
    /// Original source file location.
    pub path: Utf8PathBuf,
}

/// Marks how the asset should be processed by the SSG.
pub(crate) enum AssetKind {
    /// Data renderable to HTML. In order to process the data, a closure should be called.
    Html(Box<dyn Fn(&Sack) -> String>),
    /// Bibliographical data.
    Bibtex(Library),
    /// Image. For now they are simply cloned to the `dist` director.
    Image,
}

/// Asset corresponding to a file on disk.
pub(crate) struct Asset {
    /// The kind of a processed asset.
    pub kind: AssetKind,
    /// File metadata
    pub meta: FileItem,
}

/// Dynamically generated asset not corresponding to any file on disk. This is useful when the
/// generated page is not a content page, e.g. page list.
pub(crate) struct Virtual(Box<dyn Fn(&Sack) -> String>);

impl Virtual {
    pub fn new(call: impl Fn(&Sack) -> String + 'static) -> Self {
        Self(Box::new(call))
    }
}

/// The kind of an output item.
pub(crate) enum OutputKind {
    /// Marks an output item which corresponds to a file on disk.
    Asset(Asset),
    /// Marks an output item which doesn't correspond to any file.
    Virtual(Virtual),
}

impl From<Asset> for OutputKind {
    fn from(value: Asset) -> Self {
        OutputKind::Asset(value)
    }
}

impl From<Virtual> for OutputKind {
    fn from(value: Virtual) -> Self {
        OutputKind::Virtual(value)
    }
}

/// Renderable output
pub(crate) struct Output {
    /// The kind of an output item
    pub(crate) kind: OutputKind,
    /// Path for the output in dist
    pub(crate) path: Utf8PathBuf,
    /// Optional URL data for outputted page.
    pub(crate) link: Option<Linkable>,
}

/// Items currently in the pipeline. In order for an item to be rendered, it needs to be marked as
/// `Take`, which means it needs to have an output location assigned to itself.
pub(crate) enum PipelineItem {
    /// Unclaimed file.
    Skip(FileItem),
    /// Data ready to be processed.
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

/// This struct allows for querying the website hierarchy. It is passed to each rendered website
/// page, so that it can easily access the website metadata.
pub(crate) struct Sack<'a> {
    /// Literally all of the content
    hole: &'a [Output],
    /// Current path for the page being rendered
    path: &'a Utf8PathBuf,
    /// Original file location for this page
    file: Option<&'a Utf8PathBuf>,
}

impl<'a> Sack<'a> {
    pub fn new(hole: &'a [Output], path: &'a Utf8PathBuf, file: Option<&'a Utf8PathBuf>) -> Self {
        Self { hole, path, file }
    }

    pub fn get_links(&self, path: &str) -> Vec<LinkDate> {
        let pattern = glob::Pattern::new(path).expect("Bad glob pattern");
        self.hole
            .iter()
            .filter(|item| pattern.matches_path(item.path.as_ref()))
            .filter_map(|item| match &item.link {
                Some(Linkable::Date(link)) => Some(link.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn get_tree(&self, path: &str) -> TreePage {
        let glob = glob::Pattern::new(path).expect("Bad glob pattern");
        let list = self
            .hole
            .iter()
            .filter(|item| glob.matches_path(item.path.as_ref()))
            .filter_map(|item| match &item.link {
                Some(Linkable::Link(link)) => Some(link.clone()),
                _ => None,
            });

        let mut tree = TreePage::new();
        for link in list {
            tree.add_link(&link);
        }

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

        self.hole
            .iter()
            .filter(|item| glob.matches_path_with(item.path.as_ref(), opts))
            .filter_map(|asset| match asset.kind {
                OutputKind::Asset(ref real) => Some(real),
                _ => None,
            })
            .find_map(|asset| match asset.kind {
                AssetKind::Bibtex(ref lib) => Some(lib),
                _ => None,
            })
    }

    /// Get the path for original file location
    pub fn get_file(&self) -> Option<&'a Utf8Path> {
        self.file.map(Utf8PathBuf::as_ref)
    }
}

#[derive(Debug)]
pub(crate) struct TreePage {
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
}

pub fn gather(pattern: &str, exts: &HashSet<&'static str>) -> Vec<PipelineItem> {
    glob(pattern)
        .expect("Invalid glob pattern")
        .filter_map(|path| {
            let path = path.unwrap();
            let path = Utf8PathBuf::from_path_buf(path).expect("Filename is not valid UTF8");

            match path.is_dir() {
                true => None,
                false => Some(to_source(path, exts)),
            }
        })
        .map(Into::into)
        .collect()
}

fn to_source(path: Utf8PathBuf, exts: &HashSet<&'static str>) -> FileItem {
    let hit = path.extension().map_or(false, |ext| exts.contains(ext));

    let kind = match hit {
        true => FileItemKind::Index,
        false => FileItemKind::Bundle,
    };

    FileItem { kind, path }
}

pub fn render_all(items: &[Output]) {
    for item in items {
        let file = match &item.kind {
            OutputKind::Asset(a) => Some(&a.meta.path),
            OutputKind::Virtual(_) => None,
        };
        render(item, &Sack::new(items, &item.path, file));
    }
}

fn render(item: &Output, sack: &Sack) {
    let o = Utf8Path::new("dist").join(&item.path);
    fs::create_dir_all(o.parent().unwrap()).unwrap();

    match item.kind {
        OutputKind::Asset(ref real) => {
            let i = &real.meta.path;

            match &real.kind {
                AssetKind::Html(closure) => {
                    let mut file = File::create(&o).unwrap();
                    file.write_all(closure(sack).as_bytes()).unwrap();
                    println!("HTML: {} -> {}", i, o);
                }
                AssetKind::Bibtex(_) => {}
                AssetKind::Image => {
                    fs::create_dir_all(o.parent().unwrap()).unwrap();
                    fs::copy(i, &o).unwrap();
                    println!("Image: {} -> {}", i, o);
                }
            };
        }
        OutputKind::Virtual(Virtual(ref closure)) => {
            let mut file = File::create(&o).unwrap();
            file.write_all(closure(sack).as_bytes()).unwrap();
            println!("Virtual: -> {}", o);
        }
    }
}
