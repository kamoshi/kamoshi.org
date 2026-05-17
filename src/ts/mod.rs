mod captures;
mod configs;

use std::borrow::Cow;

use hern_doc::{Annotation, SnippetAnalysis, TextRange};
use hypertext::{Raw, prelude::*};
use tree_sitter_highlight::{HighlightEvent, Highlighter};

pub enum TSEvent {
    Write {
        start: usize,
        end: usize,
        text: String,
    },
    Enter(String),
    Close,
}

pub fn highlight(lang: &str, code: &str) -> impl Renderable {
    highlight_with_analysis(lang, code, None)
}

pub fn highlight_with_analysis(
    lang: &str,
    code: &str,
    analysis: Option<&SnippetAnalysis>,
) -> impl Renderable {
    maud!(
        figure .listing.syntax data-lang=(lang) {
            pre {
                code {
                    (Raw::dangerously_create(to_html(lang, code, analysis)))
                }
            }
        }
    )
}

fn to_html(lang: &str, code: &str, analysis: Option<&SnippetAnalysis>) -> String {
    let annotations = analysis
        .map(|analysis| analysis.annotations.as_slice())
        .unwrap_or(&[]);
    get_events(lang, code)
        .into_iter()
        .map(|event| match event {
            TSEvent::Write { start, end, text } => Cow::from(write_source_with_annotations(
                code,
                start,
                end,
                &text,
                annotations,
            )),
            TSEvent::Enter(class) => Cow::from(format!(
                "<span class=\"{}\">",
                crate::utils::escape_html_attr(&class.replace('.', "-"))
            )),
            TSEvent::Close => Cow::from("</span>"),
        })
        .collect()
}

fn get_events(lang: &str, src: &str) -> Vec<TSEvent> {
    let config = match configs::get_config(lang) {
        Some(c) => c,
        None => {
            return vec![TSEvent::Write {
                start: 0,
                end: src.len(),
                text: src.into(),
            }];
        }
    };

    let mut hl = Highlighter::new();
    let Ok(highlights) = hl.highlight(config, src.as_bytes(), None, |name| {
        configs::get_config(name)
    }) else {
        return vec![TSEvent::Write {
            start: 0,
            end: src.len(),
            text: src.into(),
        }];
    };

    let mut out = vec![];
    for event in highlights {
        let Ok(event) = event else {
            return vec![TSEvent::Write {
                start: 0,
                end: src.len(),
                text: src.into(),
            }];
        };
        let obj = map_event(event, src);
        out.push(obj);
    }
    out
}

fn map_event(event: HighlightEvent, src: &str) -> TSEvent {
    match event {
        HighlightEvent::Source { start, end } => TSEvent::Write {
            start,
            end,
            text: src[start..end].into(),
        },
        HighlightEvent::HighlightStart(s) => TSEvent::Enter(captures::NAMES[s.0].into()),
        HighlightEvent::HighlightEnd => TSEvent::Close,
    }
}

fn write_source_with_annotations(
    code: &str,
    start: usize,
    end: usize,
    text: &str,
    annotations: &[Annotation],
) -> String {
    let mut out = String::new();
    let mut cursor = start;

    let mut annotations = annotations
        .iter()
        .filter(|annotation| ranges_overlap(annotation.range, TextRange::new(start, end)))
        .collect::<Vec<_>>();
    annotations.sort_by_key(|annotation| (annotation.range.start, annotation.range.end));
    debug_assert!(
        annotations
            .windows(2)
            .all(|pair| pair[0].range.end <= pair[1].range.start)
    );

    for annotation in annotations {
        let annotation_start = annotation.range.start.max(start);
        let annotation_end = annotation.range.end.min(end);
        if annotation_start < cursor || annotation_start >= annotation_end {
            continue;
        }

        out.push_str(&crate::utils::escape_html_text(
            &text[cursor - start..annotation_start - start],
        ));
        out.push_str(&annotation_open_tag(code, annotation));
        out.push_str(&crate::utils::escape_html_text(
            &text[annotation_start - start..annotation_end - start],
        ));
        out.push_str(annotation_close_tag(annotation));
        cursor = annotation_end;
    }

    out.push_str(&crate::utils::escape_html_text(&text[cursor - start..]));
    out
}

fn annotation_open_tag(code: &str, annotation: &Annotation) -> String {
    let mut class = Vec::new();
    if annotation.hover.is_some() {
        class.push("hern-hover");
    }
    if annotation.id.is_some() {
        class.push("hern-def");
    }
    if annotation.link.is_some() {
        class.push("hern-ref");
    }

    let mut attrs = String::new();
    if !class.is_empty() {
        attrs.push_str(&format!(" class=\"{}\"", class.join(" ")));
    }
    if let Some(id) = &annotation.id {
        attrs.push_str(&format!(" id=\"{}\"", crate::utils::escape_html_attr(id)));
    }
    if let Some(hover) = &annotation.hover {
        attrs.push_str(&format!(
            " data-hern-hover=\"{}\" data-hern-highlight-start=\"{}\" data-hern-highlight-end=\"{}\"{}",
            crate::utils::escape_html_attr(hover),
            annotation.highlight.start,
            annotation.highlight.end,
            hover_placement_attr(code, annotation.range),
        ));
    }

    if let Some(link) = &annotation.link {
        format!(
            "<a href=\"#{}\"{}>",
            crate::utils::escape_html_attr(&link.target_id),
            attrs
        )
    } else {
        format!("<span{}>", attrs)
    }
}

fn annotation_close_tag(annotation: &Annotation) -> &'static str {
    if annotation.link.is_some() {
        "</a>"
    } else {
        "</span>"
    }
}

fn hover_placement_attr(code: &str, trigger: TextRange) -> &'static str {
    if !code.contains('\n') {
        // Hern ranges are byte offsets. This midpoint is only used for a
        // left/right numeric comparison, so it never indexes into the string.
        let trigger_midpoint = trigger.start + (trigger.end - trigger.start) / 2;
        if trigger_midpoint <= code.len() / 2 {
            " data-hern-placement=\"right\""
        } else {
            " data-hern-placement=\"left\""
        }
    } else if code[..trigger.start].contains('\n') {
        ""
    } else {
        " data-hern-placement=\"bottom\""
    }
}

fn ranges_overlap(lhs: TextRange, rhs: TextRange) -> bool {
    lhs.start < rhs.end && rhs.start < lhs.end
}
