use std::cell::RefCell;

use comrak::{Arena, nodes::{Ast, AstNode, LineColumn, NodeValue}};
use hayagriva::{BibliographyDriver, Library};
use once_cell::sync::Lazy;
use regex::Regex;

use super::render::iter_nodes;


static RE_CITE: Lazy<Regex> = Lazy::new(||
    Regex::new(r":cite\[(\w+)\]").unwrap()
);

pub fn add_cite<'a>(root: &'a AstNode<'a>, arena: &'a Arena<AstNode<'a>>) {
    // let mut driver = BibliographyDriver::new();

    iter_nodes(root, &|node| {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::Text(ref text) => {
                for xd in RE_CITE.captures_iter(text) {
                    let text = xd.get(1).unwrap().as_str();
                    println!("{:?}", text);
                }
            },
            _ => (),
        }
    });
}
