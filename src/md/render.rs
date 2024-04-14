use std::cell::RefCell;
use comrak::{Arena, parse_document, format_html, Options};
use comrak::nodes::{Ast, AstNode, LineColumn, NodeValue};
use once_cell::unsync::Lazy;

use crate::ts;


const OPTIONS: Lazy<Options> = Lazy::new(||
    Options {
        extension: comrak::ExtensionOptionsBuilder::default()
            .front_matter_delimiter(Some("---".into()))
            .table(true)
            .math_dollars(true)
            .shortcodes(true)
            .build()
            .unwrap(),
        parse: comrak::ParseOptionsBuilder::default()
            .smart(true)
            .build()
            .unwrap(),
        render: comrak::RenderOptionsBuilder::default()
            .unsafe_(true)
            .build()
            .unwrap(),
    }
);


pub fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
    where F : Fn(&'a AstNode<'a>) {
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}


pub fn render(raw: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, raw, &OPTIONS);

    iter_nodes(root, &|node| {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::CodeBlock(ref mut inner) => {
                let html = ts::highlight(&inner.info, &inner.literal);
                let html = hypertext::Renderable::render(html);
                let elem = AstNode::new(RefCell::new(Ast::new(NodeValue::HtmlInline(html.into()), LineColumn { line: 0, column: 0 })));
                let elem = arena.alloc(elem);
                node.insert_before(elem);
                node.detach();
            },
            &mut NodeValue::Math(ref text) => {
                let opts = katex::opts::Opts::builder()
                    .output_type(katex::OutputType::Mathml)
                    .display_mode(text.display_math)
                    .build()
                    .unwrap();
                let math = katex::render_with_opts(&text.literal, opts).unwrap();
                let elem = AstNode::new(RefCell::new(Ast::new(NodeValue::HtmlInline(math.into()), LineColumn { line: 0, column: 0 })));
                let elem = arena.alloc(elem);
                node.insert_before(elem);
                node.detach();
            },
            _ => (),
        }
    });

    super::ruby::add_ruby(root, &arena);
    super::cite::add_cite(root, &arena);

    let mut html = vec![];
    format_html(root, &OPTIONS, &mut html).unwrap();

    String::from_utf8(html).unwrap()
}

