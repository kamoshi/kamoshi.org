use hypertext::{html_elements, maud_move, GlobalAttributes, Raw, Renderable};
use crate::html::base::{head, navbar, footer};


pub fn bare<'data, 'html, R>(title: &'data str, main: R) -> impl Renderable + 'html
    where
        'data : 'html,
        R: Renderable + 'data
{
    maud_move!(
        (Raw("<!DOCTYPE html>"))
        html lang="en" {
            (head(title))

            body {
                (main)
            }
        }
    )
}

pub fn page<'data, 'main, 'page, T>(title: &'data str, main: T) -> impl Renderable + 'page
    where
        'main : 'page,
        'data : 'page,
        T: Renderable + 'main
{
    maud_move!(
        (Raw("<!DOCTYPE html>"))
        html lang="en" {
            (head(title))

            body {
                (navbar())
                (main)
                (footer())
            }
        }
    )
}
