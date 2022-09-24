use crate::{haskell, toml};
use proc_macro::TokenStream;
use quote::quote;
use std::{fs, sync::Mutex};

pub(crate) fn generate(item_fn: syn::ItemFn) -> TokenStream {
    // Parsing function declaration token stream ...
    let ident = item_fn.sig.ident.to_string();
    let c_ident = format!("c_{ident}");

    // Neat hack to keep track of all exposed functions ...
    static SIGNATURES: Mutex<Vec<haskell::Signature>> = Mutex::new(vec![]);
    let signatures = &mut *SIGNATURES.lock().unwrap();
    signatures.push(haskell::Signature { ident: ident.clone() });

    // Generate Haskell bindings into module defined in `.hsbindgen` config ...
    let module = toml::config()
        .default
        .expect("your `.hsbindgen` file should contain a `default` field");
    fs::write(
        format!("src/{module}.hs"),
        haskell::template(&module, signatures),
    )
    .unwrap();

    // Generate extra Rust code that wrap our exposed function ...
    quote!({
        #[no_mangle] // Mangling randomize symbols
        pub unsafe extern "C" fn #c_ident() {
            // FIXME: this is a trick to currently not allow function that
            // either take or return argument, indeed this should be fixed
            #ident()
        }
    })
    .into()
}
