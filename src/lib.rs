//! # `hs-bindgen-derive`
//!
//! This library define the `#[hs_bindgen]` procedural macro used by
//! [`hs-bindgen`](https://github.com/yvan-sraka/hs-bindgen) library.

use proc_macro::TokenStream;
use std::{fs, sync::Mutex};

mod haskell;
mod rust;
mod toml;

#[proc_macro_attribute]
pub fn hs_bindgen(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = input.clone();

    // Generate extra Rust code that wrap our exposed function ...
    let (signature, extern_c_wrapper) = rust::generate(
        attrs.to_string().parse().ok(),
        syn::parse(input)
            .expect("failed to parse as Rust code the content of `#[hs_bindgen]` macro"),
    );

    // Neat hack to keep track of all exposed functions ...
    static SIGNATURES: Mutex<Vec<haskell::Signature>> = Mutex::new(vec![]);
    let signatures = &mut *SIGNATURES.lock().unwrap();
    signatures.push(signature);

    // Generate Haskell bindings into module defined in `.hsbindgen` config ...
    let module = toml::config()
        .default
        .expect("your `.hsbindgen` file should contain a `default` field");
    fs::write(
        format!("src/{module}.hs"),
        haskell::template(&module, signatures),
    )
    .unwrap_or_else(|_| panic!("fail to write `src/{module}.hs` file"));

    output.extend(extern_c_wrapper);
    output
}
