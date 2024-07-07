use std::fmt::Write as _;

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    element: std::collections::HashMap<String, Element>,
    attribute: std::collections::HashMap<String, Attribute>,
}

#[derive(Deserialize)]
struct Element {
    // TODO: JS element type
}

#[derive(Deserialize)]
struct Attribute {
    type_name: Option<String>,
    value_type: Option<String>,
    value_trait: Option<String>,
    value_wrapper: Option<String>,
}

impl Attribute {
    fn value_trait(&self) -> &str {
        assert!(self.value_type.is_none());
        self.value_trait.as_deref().unwrap_or("AttrValue")
    }
}

fn main() {
    let config = std::fs::read_to_string("generate.toml").unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);

    gen_el(&config, &out_dir);
    gen_el_types(&config, &out_dir);

    gen_attr(&config, &out_dir);
}

fn gen_el_types(config: &Config, out_dir: &std::path::Path) {
    let mut src = String::new();

    src.push_str("#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#\"\n");

    for name in config.element.keys() {
        writeln!(&mut src, "export function create_{name}() {{return document.createElement(\"{name}\")}}").unwrap();
    }

    src.push_str("\"#)]\n");
    src.push_str("extern \"C\" {\n");

    for (name, Element {}) in &config.element {
        writeln!(&mut src, "fn create_{name}() -> web_sys::Element;").unwrap();
    }

    src.push_str("}\n");

    for name in config.element.keys() {
        let t = type_name(name);
        writeln!(&mut src, "make_el!({name}, {t}, create_{name}());").unwrap();
    }

    std::fs::write(out_dir.join("gen_el_types.rs"), src).unwrap();
}

fn gen_el(config: &Config, out_dir: &std::path::Path) {
    let mut src = String::new();

    for name in config.element.keys() {
        let t = type_name(name);
        // Ideally this would be generated by a macro, but rust-analyzer can't
        // seem to handle doc attributes generated by a macro generated by a
        // build script.
        writeln!(&mut src, "/// [`<{name}>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/{name}) element.").unwrap();
        writeln!(
            &mut src,
            "pub fn {name}<Body>(body: Body) -> types::{t}<Body> {{ types::{t}(body) }}"
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("gen_el.rs"), src).unwrap();

    println!("cargo::rerun-if-changed=generate.toml");
}

fn gen_attr(config: &Config, out_dir: &std::path::Path) {
    let mut src = String::new();

    // TODO: Per-attr JS snippets like for elements.

    for (name, attr) in &config.attribute {
        let t = attr.type_name.clone().unwrap_or(type_name(name));

        // Ideally this would be generated by a macro, but rust-analyzer can't
        // seem to handle doc attributes generated by a macro generated by a
        // build script.
        writeln!(&mut src, "/// `{name}` attribute.").unwrap();
        writeln!(&mut src, "#[derive(Copy, Clone)]").unwrap();

        if let Some(value_type) = &attr.value_type {
            assert!(attr.value_trait.is_none());

            writeln!(&mut src, "pub struct {t}(pub {value_type});").unwrap();

            match &attr.value_wrapper {
                Some(value_wrapper) => writeln!(
                    &mut src,
                    "make_attr_value_type!(\"{name}\", {t}, {value_type}, {value_wrapper});",
                ),
                None => writeln!(
                    &mut src,
                    "make_attr_value_type!(\"{name}\", {t}, {value_type});",
                ),
            }
            .unwrap();
        } else {
            let value_trait = attr.value_trait();

            writeln!(&mut src, "pub struct {t}<V: {value_trait}>(pub V);")
                .unwrap();

            match &attr.value_wrapper {
                Some(value_wrapper) => writeln!(
                    &mut src,
                    "make_attr_value_trait!(\"{name}\", {t}, {value_trait}, {value_wrapper});",
                ),
                None => writeln!(
                    &mut src,
                    "make_attr_value_trait!(\"{name}\", {t}, {value_trait});",
                ),
            }
            .unwrap();
        }
    }

    std::fs::write(out_dir.join("gen_attr.rs"), src).unwrap();
}

fn type_name(s: &str) -> String {
    let mut cs = s.chars();
    let mut s = String::with_capacity(s.len());

    s.push(cs.next().unwrap().to_ascii_uppercase());
    while let Some(c) = cs.next() {
        s.push(match c {
            '-' => cs.next().unwrap().to_ascii_uppercase(),
            c => c,
        });
    }

    s
}
