use std::fmt::Write as _;

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    element: std::collections::HashMap<String, Element>,
}

#[derive(Deserialize)]
struct Element {
    // TODO: JS element type
}

fn main() {
    let config = std::fs::read_to_string("generate.toml").unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);

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
        let t = title_case(name);
        writeln!(&mut src, "make_el!({name}, {t}, create_{name}());").unwrap();

        // Ideally this would be part of `make_el`, but rust-analyzer can't
        // seem to handle doc attributes generated by a macro generated by a
        // build script.
        writeln!(&mut src, "/// [`<{name}>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/{name}) element.").unwrap();
        writeln!(
            &mut src,
            "pub fn {name}<Body>(body: Body) -> {t}<Body> {{ {t}(body) }}"
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("el_gen.rs"), src).unwrap();

    println!("cargo::rerun-if-changed=generate.toml");
}

fn title_case(s: &str) -> String {
    let mut cs = s.chars();
    match cs.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
    }
}
