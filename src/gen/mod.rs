mod load;
mod sack;

use camino::Utf8PathBuf;
use hayagriva::Library;
use hypertext::Renderable;

pub use load::{gather, render_all, FileItem, FileItemKind, Asset, AssetKind, PipelineItem, Dynamic, Output};
pub use sack::{TreePage, Sack};

use crate::{html::Linkable, text::md::Outline};


/// Represents a piece of content that can be rendered as a page.
pub trait Content {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        outline: Outline,
        sack: &'s Sack,
        bib: Option<Vec<String>>,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm;

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable>;

    fn render(data: &str, lib: Option<&Library>) -> (Outline, String, Option<Vec<String>>);
}
