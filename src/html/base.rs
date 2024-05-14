use camino::Utf8Path;
use hypertext::{html_elements, maud, maud_move, GlobalAttributes, Raw, Renderable};

use crate::REPO;


const JS_RELOAD: &str = r#"
const socket = new WebSocket("ws://localhost:1337");
socket.addEventListener("message", (event) => {
    console.log(event);
    window.location.reload();
});
"#;

const JS_IMPORTS: &str = r#"
{
    "imports": {
        "splash": "/js/splash.js",
        "reveal": "/js/reveal.js",
        "photos": "/js/photos.js"
    }
}
"#;


pub fn head(title: &str) -> impl Renderable + '_ {
    let title = format!("{} | kamoshi.org", title);

    maud_move!(
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title {
            (title)
        }

        // link rel="sitemap" href="/sitemap.xml";

        link rel="stylesheet" href="/styles.css";
        link rel="stylesheet" href="/static/css/reveal.css";
        link rel="stylesheet" href="/static/css/leaflet.css";
        link rel="stylesheet" href="/static/css/MarkerCluster.css";
        link rel="stylesheet" href="/static/css/MarkerCluster.Default.css";
        link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
        link rel="icon" href="/favicon.ico" sizes="any";

        script type="importmap" {(Raw(JS_IMPORTS))}

        script { (Raw(JS_RELOAD)) }
    )
}

pub fn navbar() -> impl Renderable {
    static ITEMS: &[(&str, &str)] = &[
        ("Posts", "/posts/"),
        ("Slides", "/slides/"),
        ("Wiki", "/wiki/"),
        ("Map", "/map/"),
        ("About", "/about/"),
        ("Search", "/search/"),
    ];

    maud!(
        nav .p-nav {
            input #p-nav-toggle type="checkbox" hidden;

            div .p-nav__bar {
                a .p-nav__logo href="/" {
                    img .p-nav__logo-icon height="48px" width="51px" src="/static/svg/aya.svg" alt="";
                    div .p-nav__logo-text {
                        div .p-nav__logo-main {
                            (Raw(include_str!("logotype.svg")))
                        }
                        div #p-nav-splash .p-nav__logo-sub {
                          "夢現の遥か彼方"
                        }
                    }
                }

                label .p-nav__burger for="p-nav-toggle" tabindex="0" {
                    span .p-nav__burger-icon {}
                }
            }

            menu .p-nav__menu {
                @for (name, url) in ITEMS {
                    li .p-nav__menu-item {
                        a .p-nav__menu-link href=(*url) {
                            (*name)
                        }
                    }
                }
            }
        }
    )
}

pub fn footer(path: Option<&Utf8Path>) -> impl Renderable {
    let copy = format!("Copyright &copy; {} Maciej Jur", &REPO.year);
    let mail = "maciej@kamoshi.org";
    let href = format!("mailto:{}", mail);
    let link = Utf8Path::new(&REPO.link).join("src/commit").join(&REPO.hash);
    let link = match path {
        Some(path) => link.join(path),
        None       => link,
    };

    maud_move!(
        footer .footer {
            div .left {
                div {
                    (Raw(copy))
                }
                a href=(href)  {
                    (mail)
                }
            }
            div .repo {
                a href=(link.as_str()) {
                    (&REPO.hash)
                }
                div {
                    (&REPO.date)
                }
            }
            a .right.footer__cc-wrap rel="license" href="http://creativecommons.org/licenses/by/4.0/" {
                img .footer__cc-stamp alt="Creative Commons License" width="88" height="31" src="/static/svg/by.svg";
            }
        }
    )
}
