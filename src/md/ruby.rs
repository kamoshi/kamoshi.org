use std::cell::RefCell;

use comrak::{Arena, nodes::{Ast, AstNode, LineColumn, NodeValue}};
use once_cell::unsync::Lazy;
use regex::Regex;

use super::render::iter_nodes;


const RE_RUBY: Lazy<Regex> = Lazy::new(||
    Regex::new(r"\[([^\]]+)\]\{([^}]+)\}").unwrap()
);

#[derive(Debug)]
enum Annotated<'a> {
    Text(&'a str),
    Ruby(&'a str, &'a str),
}


pub fn add_ruby<'a>(root: &'a AstNode<'a>, arena: &'a Arena<AstNode<'a>>) {
    iter_nodes(root, &|node| {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::Text(ref text) => {
                for item in annotate(text) {
                    let new = match item {
                        Annotated::Text(text) => NodeValue::Text(text.into()),
                        Annotated::Ruby(t, f) => NodeValue::HtmlInline(format!("<ruby>{t}<rp>(</rp><rt>{f}</rt><rp>)</rp></ruby>")),
                    };
                    let elem = AstNode::new(RefCell::new(Ast::new(new, LineColumn { line: 0, column: 0 })));
                    let elem = arena.alloc(elem);
                    node.insert_before(elem)
                }
                node.detach();
            },
            _ => (),
        }
    });
}

fn annotate(input: &str) -> Vec<Annotated> {
    let mut parts: Vec<Annotated> = Vec::new();
    let mut last_index = 0;

    for cap in RE_RUBY.captures_iter(input) {
        let text = cap.get(1).unwrap().as_str();
        let ruby = cap.get(2).unwrap().as_str();
        let index = cap.get(0).unwrap().start();

        if index > last_index {
            parts.push(Annotated::Text(&input[last_index..index]));
        }

        parts.push(Annotated::Ruby(text, ruby));
        last_index = cap.get(0).unwrap().end();
    }

    if last_index < input.len() {
        parts.push(Annotated::Text(&input[last_index..]));
    }

    parts
}
