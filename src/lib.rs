//! # `hs-bindgen`
//!
//! Handy macro to generate C-FFI bindings from Rust to Haskell.
//!
//! This library intended to work best in a project configured by
//! [`cabal-pack`](https://github.com/yvan-sraka/cabal-pack).
//!
//! ## Acknowledgments
//!
//! ⚠️ This is still a working experiment, not yet production ready.
//!
//! `hs-bindgen` was heavily inspired by other interoperability initiatives, as
//! [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) and
//! [`PyO3`](https://github.com/PyO3/pyo3).
//!
//! This project was part of a work assignment as an
//! [IOG](https://github.com/input-output-hk) contractor.

use proc_macro::TokenStream;
use std::{fs, sync::Mutex};

mod haskell;
mod rust;
mod toml;

#[proc_macro_attribute]
pub fn hs_bindgen(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // FIXME: macro attributes would be used to customize, exposed C FFI
    // function name, or the path Haskell module where binding are generated
    let mut output = input.clone();

    // Generate extra Rust code that wrap our exposed function ...
    let (signature, extern_c_wrapper) = rust::generate(
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
