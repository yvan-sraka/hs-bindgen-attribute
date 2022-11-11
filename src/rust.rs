use crate::{antlion, haskell};
use hs_bindgen_traits::HsType;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

/// Generate extra Rust code that wrap our exposed function
pub(crate) fn generate(
    attrs: TokenStream,
    item_fn: syn::ItemFn,
) -> (haskell::Signature, TokenStream) {
    let rust_fn = format_ident!("{}", item_fn.sig.ident.to_string());

    // Parse targeted Haskell function signature either from proc macro
    // attributes or either from types from Rust `fn` item (using feature
    // `antlion` which is enabled by default) ...
    let mut sig = {
        let s = attrs.to_string();
        if cfg!(feature = "antlion") && s.is_empty() {
            let sig = <haskell::Signature as antlion::Eval<&syn::ItemFn>>::from(&item_fn);
            antlion::warning(&sig);
            sig
        } else {
            s.parse().unwrap_or_else(|e| panic!("{e}"))
        }
    };

    // Ensure that Haskell signature end by `IO` type ...
    const UNSUPPORTED_RETURN_TYPE: &str =
        "`hs-bindgen` currently only support Haskell function signature that end by returning `IO`";
    let ret = match sig.fn_type.pop().expect(UNSUPPORTED_RETURN_TYPE) {
        HsType::IO(x) => x,
        _ => panic!("{UNSUPPORTED_RETURN_TYPE}"),
    };

    // Iterate through function argument types ...
    let mut c_fn_args = quote! {};
    let mut rust_fn_values = quote! {};
    for (i, hs_c_ffi_type) in sig.fn_type.iter().enumerate() {
        let arg = format_ident!("__{i}");
        let c_ffi_safe_type = hs_c_ffi_type.quote();
        c_fn_args.extend(quote! { #arg: #c_ffi_safe_type, });
        rust_fn_values.extend(quote! { traits::ReprRust::from(#arg), });
    }

    // Generate C-FFI wrapper of Rust function ...
    let c_fn = format_ident!("__c_{}", sig.fn_name);
    let c_ret = ret.quote();
    let extern_c_wrapper = quote! {
        #[no_mangle] // Mangling randomize symbols
        extern "C" fn #c_fn(#c_fn_args) -> #c_ret {
            // `traits` module is `hs-bindgen::hs-bindgen-traits`
            // n.b. do not forget to import it, e.g., with `use hs-bindgen::*`
            traits::ReprC::from(#rust_fn(#rust_fn_values))
        }
    };

    // DEBUG: println!("{extern_c_wrapper}");
    sig.fn_type.push(HsType::IO(ret));
    (sig, extern_c_wrapper.into())
}
