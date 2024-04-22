mod load;
mod render;
mod sack;

use camino::Utf8PathBuf;
use hypertext::Renderable;

pub use load::{gather, Source, SourceKind};
pub use render::{render, Asset, AssetKind, Virtual, Item};
pub use sack::{TreePage, Sack};

use crate::{html::Linkable, text::md::Outline};


/// Represents a piece of content that can be rendered into a page.
pub trait Content {
    fn transform<'f, 'm, 's, 'html, T>(
        &'f self,
        content: T,
        outline: Outline,
        sack: &'s Sack,
    ) -> impl Renderable + 'html
        where
            'f: 'html,
            'm: 'html,
            's: 'html,
            T: Renderable + 'm;

    fn as_link(&self, path: Utf8PathBuf) -> Option<Linkable>;

    fn render(data: &str) -> (Outline, String);
}
