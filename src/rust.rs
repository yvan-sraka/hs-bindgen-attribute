use crate::haskell;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

/// Generate extra Rust code that wrap our exposed function
///
/// # Example
///
/// ```
/// use hs_bindgen::hs_bindgen;
///
/// #[hs_bindgen]
/// fn greetings(name: &str) {
///     println!("Hello, {name}!");
/// }
/// ```
///
/// ... will be expanded to ...
///
/// ```
/// fn greetings(name: &str) {
///     println!("Hello, {name}!");
/// }
///
/// #[no_mangle] // Mangling randomize symbols
/// pub unsafe extern "C" fn c_greetings(_0: *const std::os::raw::c_char) {
///     let _0 = std::ffi::CStr::from_ptr(_0).to_str().unwrap();
///     greetings(_0)
/// }
/// ```
pub(crate) fn generate(item_fn: syn::ItemFn) -> (haskell::Signature, TokenStream) {
    // Parsing function declaration token stream ...
    let fn_name = item_fn.sig.ident.to_string();
    let rust_fn = format_ident!("{fn_name}");
    let c_fn = format_ident!("c_{fn_name}");
    // FIXME: this whole routine of type matching should better live in a
    // separate function or module!
    let mut hs_fn_type = vec![];
    let mut c_fn_args = quote! {};
    let mut rust_fn_values = quote! {};
    for (i, input) in (&item_fn.sig.inputs).into_iter().enumerate() {
        let arg = format_ident!("_{i}");
        let ty = match input {
            syn::FnArg::Typed(p) => &*p.ty,
            _ => panic!("functions using `self` are not supported by `hs-bindgen`"),
        };
        let ty = quote! { #ty }.to_string();
        let (ty, cast, hs) = match ty.as_str() {
            "String" => (
                quote! { *const std::ffi::c_char },
                quote! { std::ffi::CStr::from_ptr(#arg).to_str().unwrap().to_string() },
                "CString",
            ),
            "& str" => (
                quote! { *const std::ffi::c_char },
                quote! { std::ffi::CStr::from_ptr(#arg).to_str().unwrap() },
                "CString",
            ),
            // FIXME: add more primitives supported types!
            _ => panic!(
                "auto-cast from type {ty} isn't implemented in `hs-bindgen`
consider opening an issue https://github.com/yvan-sraka/hs-bindgen"
            ),
        };
        c_fn_args.extend(quote! { #arg: #ty, });
        rust_fn_values.extend(quote! { #cast, });
        hs_fn_type.push(hs.to_owned());
    }
    (
        haskell::Signature {
            fn_name,
            fn_type: hs_fn_type,
        },
        quote! {
            #[no_mangle] // Mangling randomize symbols
            pub unsafe extern "C" fn #c_fn(#c_fn_args) {
                // FIXME: this is a trick to currently not allow function that
                // return argument, indeed this should be fixed
                #rust_fn(#rust_fn_values)
            }
        }
        .into(),
    )
}
