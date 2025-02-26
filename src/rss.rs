use camino::{Utf8Path, Utf8PathBuf};
use hauchiwa::QueryContent;
use rss::{ChannelBuilder, ItemBuilder};

use crate::model::Post;
use crate::{BASE_URL, MySack, Slideshow};

pub(crate) trait ToFeed: Sized {
	fn to_feed(query: QueryContent<Self>) -> rss::Item;
}

impl ToFeed for Post {
	fn to_feed(query: QueryContent<Self>) -> rss::Item {
		ItemBuilder::default()
			.title(query.meta.title.clone())
			.link(
				Utf8Path::new(BASE_URL)
					.join(query.slug.to_string())
					.to_string(),
			)
			.build()
	}
}

impl ToFeed for Slideshow {
	fn to_feed(query: QueryContent<Self>) -> rss::Item {
		ItemBuilder::default()
			.title(query.meta.title.clone())
			.link(
				Utf8Path::new(BASE_URL)
					.join(query.slug.to_string())
					.to_string(),
			)
			.build()
	}
}

pub(crate) fn generate_feed<T: ToFeed + 'static>(
	sack: MySack,
	slug: &str,
	title: &str,
) -> Vec<(Utf8PathBuf, String)> {
	let slug = Utf8Path::new(slug);
	let glob = slug.join("**/*");

	vec![(
		slug.join("rss.xml"),
		ChannelBuilder::default()
			.title(title)
			.link(Utf8Path::new(BASE_URL).join(slug).to_string())
			.items(
				sack.query_content::<T>(glob.as_str())
					.into_iter()
					.map(T::to_feed)
					.collect::<Vec<_>>(),
			)
			.build()
			.to_string(),
	)]
}
