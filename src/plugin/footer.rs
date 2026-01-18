use camino::Utf8Path;
use hypertext::{Renderable, maud, prelude::*};

use crate::Context;

const MAIL_A: &str = "maciej";
const MAIL_B: &str = "kamoshi.org";

pub fn render(ctx: &Context) -> impl Renderable {
    let year = &ctx.env.data.year;
    let mail_href = format!("mailto:{}@{}", MAIL_A, MAIL_B);

    let repo_link = Utf8Path::new(&ctx.env.data.link)
        .join("tree")
        .join(&ctx.env.data.hash);
    let hash_short = &ctx.env.data.hash[0..7];
    let date = &ctx.env.data.date;

    maud!(
        footer .footer {
            img .kaeru src="/static/svg/choju/敷物を持つカエル.svg" alt="Decoration";

            div .left {
                 div .copyright { "© " (year) " Maciej Jur" }
                 a href=(mail_href) { (MAIL_A) "@" (MAIL_B) }
            }

            // license stamp
            a .cc rel="license" href="http://creativecommons.org/licenses/by/4.0/" {
                img alt="Creative Commons License" src="/static/svg/by.svg";
            }

            div .right {
                div { "Build: " a .hash href=(repo_link.as_str()) { (hash_short) } }
                div { "(" (date) ")" }
            }

            img .neko src="/static/svg/choju/猫貴族.svg" alt="Decoration";
        }
    )
}
