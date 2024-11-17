extern crate proc_macro;
use lightningcss::{declaration::DeclarationBlock, traits::ToCss};
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn style(input: TokenStream) -> TokenStream {
    let input = input.to_string();

    let styles = DeclarationBlock::parse_string(&input, Default::default())
        .expect("Failed to parse CSS");

    let minified = styles
        .to_css_string(lightningcss::printer::PrinterOptions {
            minify: true,
            ..Default::default()
        })
        .expect("Failed to minify CSS");

    quote! {
        ::ravel_web::attr::Style(#minified)
    }
    .into()
}
