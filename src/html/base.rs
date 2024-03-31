use chrono::{self, Datelike};
use hypertext::{html_elements, maud, maud_move, GlobalAttributes, Raw, Renderable};


pub fn head(title: &str) -> impl Renderable + '_ {
    maud_move!(
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title {
            (title)
        }

        script type="importmap" {(Raw(r#"
            {
                "imports": {
                    "splash": "/js/splash.js",
                    "reveal": "/js/reveal.js",
                    "photos": "/js/photos.js"
                }
            }
        "#))}

        // link rel="sitemap" href="/sitemap.xml";

        link rel="stylesheet" href="/styles.css";
        link rel="stylesheet" href="/static/css/reveal.css";
        link rel="stylesheet" href="/static/css/leaflet.css";
        link rel="stylesheet" href="/static/css/MarkerCluster.css";
        link rel="stylesheet" href="/static/css/MarkerCluster.Default.css";
        link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
        link rel="icon" href="/favicon.ico" sizes="any";
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
                          "Doesn't require JavaScript!"
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

            script type="module" {
                (Raw("import 'splash';"))
            }
        }
    )
}

pub fn footer() -> impl Renderable {
    let year = chrono::Utc::now().year();
    let copy = format!("Copyright &copy; {} Maciej Jur", year);
    let mail = "maciej@kamoshi.org";
    let href = format!("mailto:{}", mail);

    maud_move!(
        footer .footer {
            div {
                div {
                    (Raw(copy))
                }
                a href=(href)  {
                    (mail)
                }
            }
            a .footer__cc-wrap rel="license" href="http://creativecommons.org/licenses/by/4.0/" {
                img .footer__cc-stamp alt="Creative Commons License" width="88" height="31" src="/static/svg/by.svg";
            }
        }
    )
}
