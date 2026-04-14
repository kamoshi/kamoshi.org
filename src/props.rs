use minijinja::Value;
use serde::Serialize;

#[derive(Serialize)]
pub struct PropsHead {
    pub title: String,
    pub generator: &'static str,
    pub importmap: Value,
    pub styles: Vec<String>,
    pub scripts: Vec<String>,
    pub refresh_script: Option<Value>,
}

#[derive(Serialize)]
pub struct PropsNavItem {
    pub stamp: &'static str,
    pub name: &'static str,
    pub url: &'static str,
}

#[derive(Serialize)]
pub struct PropsNavbar {
    pub logotype_svg: Value,
    pub items: Vec<PropsNavItem>,
}

#[derive(Serialize)]
pub struct PropsFooter {
    pub year: i32,
    pub repo_link: String,
    pub hash_short: String,
    pub date: String,
}

#[derive(Serialize)]
pub struct PropsHome {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub article: Value,
    pub intro: Value,
}

#[derive(Serialize)]
pub struct PropsPostUpdated {
    pub date: String,
    pub date_iso: String,
    pub hash: String,
    pub hash_url: String,
}

#[derive(Serialize)]
pub struct PropsPostMeta {
    pub date_added: String,
    pub date_added_iso: String,
    pub updated: Option<PropsPostUpdated>,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct PropsBibliography {
    pub items: Vec<Value>,
    pub library_path: Option<String>,
}

#[derive(Serialize)]
pub struct PropsPost {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub title: String,
    pub outline: Value,
    pub content: Value,
    pub bibliography: Option<PropsBibliography>,
    pub metadata: PropsPostMeta,
}

#[derive(Serialize)]
pub struct PropsAbout {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub title: String,
    pub outline: Value,
    pub content: Value,
    pub pubkey_ident_fingerprint: String,
    pub pubkey_email_fingerprint: String,
}

#[derive(Serialize)]
pub struct PropsMicroblogEntry {
    pub body: Value,
    pub date_iso: String,
    pub date_display: String,
    pub timestamp: i64,
}

#[derive(Serialize)]
pub struct PropsThoughts {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub entries: Vec<PropsMicroblogEntry>,
}

#[derive(Serialize)]
pub struct PropsThought {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub entry: PropsMicroblogEntry,
}

#[derive(Serialize)]
pub struct PropsWikiTreeNode {
    pub href: String,
    pub name: String,
    pub is_link: bool,
    pub is_current: bool,
    pub children: Vec<PropsWikiTreeNode>,
}

#[derive(Serialize)]
pub struct PropsWikiBacklink {
    pub href: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct PropsWiki {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub title: String,
    pub tree: Vec<PropsWikiTreeNode>,
    pub content: Value,
    pub bibliography: Option<Vec<Value>>,
    pub backlinks: Option<Vec<PropsWikiBacklink>>,
}

#[derive(Serialize)]
pub struct PropsWikiPdf {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub tree: Vec<PropsWikiTreeNode>,
    pub pdf_path: String,
}

#[derive(Serialize)]
pub struct PropsTag {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub title: String,
    pub groups: Vec<PropsListGroup>,
}

#[derive(Serialize)]
pub struct PropsTagCloudEntry {
    pub tag: String,
    pub count: usize,
}

#[derive(Serialize)]
pub struct PropsTagCloud {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub title: String,
    pub entries: Vec<PropsTagCloudEntry>,
}

#[derive(Serialize)]
pub struct PropsSlideshow {
    pub head: PropsHead,
    pub slides: Value,
}

#[derive(Serialize)]
pub struct PropsMap {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
}

#[derive(Serialize)]
pub struct PropsSearch {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
}

#[derive(Serialize)]
pub struct PropsBare {
    pub head: PropsHead,
    pub content: Value,
}

#[derive(Serialize)]
pub struct PropsProjectTile {
    pub title: String,
    pub tech: Vec<String>,
    pub link: String,
    pub desc: Option<String>,
    pub external: bool,
}

#[derive(Serialize)]
pub struct PropsProjects {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub projects: Vec<PropsProjectTile>,
}

#[derive(Serialize)]
pub struct PropsProjectPage {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub outline: Value,
    pub content: Value,
}

#[derive(Serialize)]
pub struct PropsRawPage {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub content: Value,
    pub hide_footer: bool,
}

#[derive(Serialize)]
pub struct PropsListItem {
    pub path: String,
    pub name: String,
    pub desc: Option<String>,
    pub date: String,
    pub date_iso: String,
}

#[derive(Serialize)]
pub struct PropsListGroup {
    pub year: i32,
    pub items: Vec<PropsListItem>,
}

#[derive(Serialize)]
pub struct PropsList {
    pub head: PropsHead,
    pub navbar: PropsNavbar,
    pub footer: PropsFooter,
    pub title: String,
    pub rss: &'static str,
    pub icon_rss: Value,
    pub groups: Vec<PropsListGroup>,
}
