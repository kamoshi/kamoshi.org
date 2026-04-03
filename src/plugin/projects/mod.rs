mod radicals;

use camino::Utf8PathBuf;
use hauchiwa::error::{HauchiwaError, RuntimeError};
use hauchiwa::loader::{Script, Stylesheet, TemplateEnv};
use hauchiwa::prelude::*;
use hypertext::prelude::*;
use minijinja::Value;

use crate::md::Parsed;
use crate::model::Project;
use crate::props::{PropsProjectPage, PropsProjectTile, PropsProjects, PropsRawPage};
use crate::{Context, Global};

pub struct ProjectView<'a> {
    pub title: &'a str,
    pub tech: Vec<String>,
    pub link: String,
    pub desc: Option<&'a str>,
    pub external: bool,
}

pub fn add_projects(
    config: &mut Blueprint<Global>,
    templates: One<TemplateEnv>,
    styles: Many<Stylesheet>,
    scripts: Many<Script>,
) -> Result<One<Vec<Output>>, HauchiwaError> {
    let docs = config
        .load_documents::<Project>()
        .source("content/projects/**/*.md")
        .source("content/projects/**/*.html")
        .offset("content")
        .register()?;

    let page_radicals = radicals::build(config, templates, styles)?;

    let task = config.task().using((templates, docs, styles, scripts, page_radicals)).merge(
        |ctx, (templates, docs, styles, scripts, page_radicals)| {
            let styles_list = &[
                styles.get("styles/styles.scss")?,
                styles.get("styles/layouts/projects.scss")?,
            ];

            let mut project_views: Vec<ProjectView> = vec![];
            let mut pages = vec![];

            for doc in docs.values() {
                let (link, external) = match &doc.matter.link {
                    Some(url) => (url.clone(), true),
                    None => {
                        let colocated_key = doc.meta.path.with_file_name("main.ts");
                        let mut js: Vec<&Script> = scripts
                            .get(colocated_key.as_str())
                            .ok()
                            .into_iter()
                            .collect();

                        if let Some(entries) = &doc.matter.scripts {
                            for entry in entries {
                                let key = format!("scripts/{}", entry);
                                js.push(scripts.get(&key)?);
                            }
                        }

                        let html = if doc.meta.path.extension() == Some("html") {
                            render_raw_page(ctx, templates, &doc.matter.title, &doc.text, styles_list, &js)?
                        } else {
                            let parsed = crate::md::parse(
                                &doc.text,
                                &doc.meta,
                                None,
                                None,
                                None,
                            )?;
                            render_page(ctx, templates, &doc.matter.title, &parsed, styles_list, &js)?
                        };

                        let href = doc.meta.href.clone();
                        pages.push(
                            doc.output()
                                .strip_prefix("content")?
                                .html()
                                .content(html),
                        );
                        (href, false)
                    }
                };
                project_views.push(ProjectView {
                    title: &doc.matter.title,
                    tech: doc.matter.tech.clone(),
                    link,
                    desc: doc.matter.desc.as_deref(),
                    external,
                });
            }

            project_views.push(ProjectView {
                title: "Constellations",
                tech: vec!["Svelte".into(), "TypeScript".into()],
                link: Utf8PathBuf::from("/")
                    .join(&page_radicals.path)
                    .parent()
                    .unwrap()
                    .to_string(),
                desc: Some("Try adding kanji you know and see how they connect to each other."),
                external: false,
            });

            {
                let docs_vec = docs.values().collect::<Vec<_>>();
                pages.push(crate::rss::generate_feed(
                    &docs_vec,
                    "projects",
                    "Kamoshi.org Projects",
                ));
            }

            {
                let list = render_list(ctx, templates, project_views, styles_list)?;
                pages.push(Output::html("projects", list));
            }

            Ok(pages)
        },
    );

    Ok(task)
}

pub fn render_list(
    ctx: &Context,
    templates: &TemplateEnv,
    mut projects: Vec<ProjectView>,
    styles: &[&Stylesheet],
) -> Result<String, RuntimeError> {
    projects.sort_unstable_by(|a, b| a.title.cmp(b.title));

    let props = PropsProjects {
        head: super::make_props_head(ctx, "Projects".to_string(), styles, &[])?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        projects: projects
            .iter()
            .map(|p| PropsProjectTile {
                title: p.title.to_string(),
                tech: p.tech.clone(),
                link: p.link.clone(),
                desc: p.desc.map(str::to_string),
                external: p.external,
            })
            .collect(),
    };

    let tmpl = templates.get_template("projects.jinja")?;
    Ok(tmpl.render(&props)?)
}

pub fn render_page(
    ctx: &Context,
    templates: &TemplateEnv,
    title: &str,
    parsed: &Parsed,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<String, RuntimeError> {
    let outline_html = parsed.outline.render().into_inner();

    let props = PropsProjectPage {
        head: super::make_props_head(ctx, title.to_string(), styles, scripts)?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        outline: Value::from_safe_string(outline_html),
        content: Value::from_safe_string(parsed.html.clone()),
    };

    let tmpl = templates.get_template("project_page_md.jinja")?;
    Ok(tmpl.render(&props)?)
}

pub fn render_raw_page(
    ctx: &Context,
    templates: &TemplateEnv,
    title: &str,
    html: &str,
    styles: &[&Stylesheet],
    scripts: &[&Script],
) -> Result<String, RuntimeError> {
    let props = PropsRawPage {
        head: super::make_props_head(ctx, title.to_string(), styles, scripts)?,
        navbar: super::make_props_navbar(),
        footer: super::make_props_footer(ctx),
        content: Value::from_safe_string(html.to_string()),
    };

    let tmpl = templates.get_template("project_page_html.jinja")?;
    Ok(tmpl.render(&props)?)
}
