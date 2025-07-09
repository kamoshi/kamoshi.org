use std::{collections::HashSet, fs};

use camino::Utf8Path;
use hauchiwa::{Page, TaskResult};
use pagefind::{api::PagefindIndex, options::PagefindServiceConfig};
use sitemap_rs::{
    url::{ChangeFrequency, Url},
    url_set::UrlSet,
};
use tokio::runtime::Builder;

pub fn build_pagefind(pages: &[&Page]) -> TaskResult<()> {
    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(index_pages(pages))?;

    Ok(())
}

async fn index_pages(pages: &[&Page]) -> TaskResult<()> {
    let config = PagefindServiceConfig::builder().build();
    let mut index = PagefindIndex::new(Some(config))?;

    for page in pages {
        if let Some("html") = page.path.extension() {
            index
                .add_html_file(Some(page.path.to_string()), None, page.text.to_string())
                .await?;
        }
    }

    let _ = index.write_files(Some("dist/pagefind".into())).await?;

    Ok(())
}

pub fn build_sitemap(pages: &[&Page]) -> TaskResult<()> {
    let urls = pages
        .iter()
        .map(|page| &page.path)
        .collect::<HashSet<_>>()
        .iter()
        .map(|path| {
            Url::builder(Utf8Path::new("/").join(path).parent().unwrap().to_string())
                .change_frequency(ChangeFrequency::Monthly)
                .priority(0.8)
                .build()
                .expect("failed a <url> validation")
        })
        .collect::<Vec<_>>();

    let urls = UrlSet::new(urls).expect("failed a <urlset> validation");
    let mut buf = Vec::<u8>::new();
    urls.write(&mut buf).expect("failed to write XML");

    fs::create_dir_all("dist")?;
    let mut file = fs::File::create("dist/sitemap.xml")?;
    std::io::Write::write_all(&mut file, &buf)?;

    Ok(())
}
