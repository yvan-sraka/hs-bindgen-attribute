use crate::haskell;
use antlion::Sandbox;
use hs_bindgen_traits::HsType;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

/// Generate extra Rust code that wrap our exposed function
pub(crate) fn generate(item_fn: syn::ItemFn) -> (haskell::Signature, TokenStream) {
    // Parse function declaration token stream ...
    let fn_name = item_fn.sig.ident.to_string();
    let rust_fn = format_ident!("{fn_name}");
    let c_fn = format_ident!("c_{fn_name}");

    // Iterate through function argument types ...
    let mut fn_type = vec![];
    let mut c_fn_args = quote! {};
    let mut rust_fn_values = quote! {};
    for (i, input) in (&item_fn.sig.inputs).into_iter().enumerate() {
        let arg = format_ident!("_{i}");
        let ty = match input {
            syn::FnArg::Typed(p) => &p.ty,
            _ => panic!("functions using `self` are not supported by `hs-bindgen`"),
        };
        let sandbox = Sandbox::new()
            .unwrap()
            .deps(&["hs-bindgen-traits@0.4"])
            .unwrap();
        let hs_c_ffi_type: HsType = sandbox
            .eval(quote! {
                <#ty as hs_bindgen_traits::ReprHs>::into()
            })
            .unwrap_or_else(|_| {
                panic!(
                    "auto-cast from type `{}` isn't implemented in `hs-bindgen`
consider opening an issue https://github.com/yvan-sraka/hs-bindgen",
                    quote! { #ty }
                )
            });
        let c_ffi_safe_type = hs_c_ffi_type.quote();
        c_fn_args.extend(quote! { #arg: #c_ffi_safe_type, });
        rust_fn_values.extend(quote! { hs_bindgen_traits::ReprC::from(#arg), });
        fn_type.push(hs_c_ffi_type);
    }
    let extern_c_wrapper = quote! {
        #[no_mangle] // Mangling randomize symbols
        extern "C" fn #c_fn(#c_fn_args) {
            // FIXME: this is a trick to currently not allow function that
            // return argument, indeed this should be fixed
            #rust_fn(#rust_fn_values)
        }
    };

    // DEBUG: println!("{extern_c_wrapper}");

    // FIXME: this is required by e.g. usage of Haskell `newCString`
    fn_type.push(HsType::IO(Box::new(HsType::Empty)));

    (
        haskell::Signature { fn_name, fn_type },
        extern_c_wrapper.into(),
    )
}
