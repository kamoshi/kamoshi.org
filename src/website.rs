use std::rc::Rc;

use crate::{build, pipeline::Output, watch, BuildContext, Source};

#[derive(Debug)]
pub(crate) struct Website {
	sources: Vec<Source>,
	special: Vec<Rc<Output>>,
}

impl Website {
	pub(crate) fn designer() -> WebsiteDesigner {
		WebsiteDesigner::default()
	}

	pub(crate) fn build(&self, ctx: &BuildContext) {
		let _ = build(ctx, &self.sources, &self.special.clone());
	}

	pub(crate) fn watch(&self, ctx: &BuildContext) {
		let state = build(&ctx, &self.sources, &self.special.clone());
		watch::watch(&ctx, &self.sources, state).unwrap()
	}
}

#[derive(Debug, Default)]
pub(crate) struct WebsiteDesigner {
	sources: Vec<Source>,
	special: Vec<Rc<Output>>,
}

impl WebsiteDesigner {
	pub(crate) fn add_source(mut self, source: Source) -> WebsiteDesigner {
		self.sources.push(source);
		self
	}

	pub(crate) fn add_output(mut self, output: Output) -> WebsiteDesigner {
		self.special.push(Rc::new(output));
		self
	}

	pub(crate) fn finish(self) -> Website {
		Website {
			sources: self.sources,
			special: self.special,
		}
	}
}
