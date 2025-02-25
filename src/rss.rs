use camino::{Utf8Path, Utf8PathBuf};
use rss::{ChannelBuilder, ItemBuilder};

use crate::MySack;
use crate::model::Post;

pub(crate) fn generate_feed(sack: MySack) -> Vec<(Utf8PathBuf, String)> {
	let base = Utf8Path::new("https://kamoshi.org/");

	vec![(
		"posts/rss.xml".into(),
		ChannelBuilder::default()
			.title("kamoshi.org posts")
			.link("https://kamoshi.org/posts")
			.items(
				sack.query_content::<Post>("posts/**/*")
					.into_iter()
					.map(|post| {
						ItemBuilder::default()
							.title(post.meta.title.clone())
							.link(base.join(post.slug.to_string()).to_string())
							.build()
					})
					.collect::<Vec<_>>(),
			)
			.build()
			.to_string(),
	)]
}
