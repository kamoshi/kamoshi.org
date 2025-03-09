use std::time::Instant;

use camino::Utf8PathBuf;
use hauchiwa::TaskResult;
use pagefind::api::PagefindIndex;
use pagefind::options::PagefindServiceConfig;
use tokio::runtime::Builder;

pub fn build_pagefind(pages: &[&(Utf8PathBuf, String)]) -> TaskResult<()> {
    println!("Indexing pages...");
    let start = Instant::now();

    Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(index_pages(pages))?;

    let duration = start.elapsed();
    println!("Indexed {} in {:.2?}", pages.len(), duration);

    Ok(())
}

async fn index_pages(pages: &[&(Utf8PathBuf, String)]) -> TaskResult<()> {
    let config = PagefindServiceConfig::builder().build();
    let mut index = PagefindIndex::new(Some(config))?;

    for (path, data) in pages {
        if let Some("html") = path.extension() {
            index
                .add_html_file(Some(path.to_string()), None, data.to_string())
                .await?;
        }
    }

    let _ = index.write_files(Some("dist/pagefind".into())).await?;

    Ok(())
}
