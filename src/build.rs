use std::fs;


pub fn build_styles() {
    let css = grass::from_path("styles/styles.scss", &grass::Options::default()).unwrap();
    fs::write("dist/styles.css", css).unwrap();
}
