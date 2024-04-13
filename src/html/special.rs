use hypertext::{html_elements, maud, GlobalAttributes, Raw, Renderable};

use super::page;


pub fn map() -> impl Renderable {
    page("Map", maud!(
        main {
            div #map style="height: 100%; width: 100%" {}

            script type="module" {
                (Raw("import 'photos';"))
            }
        }
    ))
}

pub fn search() -> impl Renderable {
    page("Search", maud!(
        main {

        }
    ))
}
