use crate::haskell;
use proc_macro::TokenStream;
use proc_macro2::Ident;
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
        let CFFIStub {
            c_ffi_safe_type,
            rust_cast_to_c_ffi,
            hs_c_ffi_type,
        } = get_types(&arg, ty);
        c_fn_args.extend(quote! { #arg: #c_ffi_safe_type, });
        rust_fn_values.extend(quote! { #rust_cast_to_c_ffi, });
        fn_type.push(hs_c_ffi_type);
    }
    let extern_c_wrapper = quote! {
        #[no_mangle] // Mangling randomize symbols
        pub unsafe extern "C" fn #c_fn(#c_fn_args) {
            // FIXME: this is a trick to currently not allow function that
            // return argument, indeed this should be fixed
            #rust_fn(#rust_fn_values)
        }
    };

    // DEBUG: println!("{extern_c_wrapper}");

    // FIXME: this is required by e.g. usage of Haskell `newCString`
    fn_type.push("IO ()".to_owned());

    (
        haskell::Signature { fn_name, fn_type },
        extern_c_wrapper.into(),
    )
}

/// Handy data structure that represente Rust -> C -> Haskell type casting
struct CFFIStub {
    /// C-FFI safe type, as written is Rust, valid in an `extern C` block
    c_ffi_safe_type: proc_macro2::TokenStream,
    /// Rust routine to cast a given type to `c_ffi_safe_type`
    rust_cast_to_c_ffi: proc_macro2::TokenStream,
    /// Haskell type name matching `c_ffi_safe_type` memory layout
    hs_c_ffi_type: String,
}

/// Extract from a rust `arg: ty` expression all types information needed to generate a safe C-FFI
fn get_types(arg: &Ident, ty: &syn::Type) -> CFFIStub {
    let ty = quote! { #ty }.to_string();
    let (c_ffi_safe_type, rust_cast_to_c_ffi, hs_c_ffi_type) = match ty.as_str() {
        "String" => (
            quote! { *const std::os::raw::c_char },
            quote! { std::ffi::CStr::from_ptr(#arg).to_str().unwrap().to_string() },
            "CString".to_owned(),
        ),
        "& str" => (
            quote! { *const std::os::raw::c_char },
            quote! { std::ffi::CStr::from_ptr(#arg).to_str().unwrap() },
            "CString".to_owned(),
        ),
        // FIXME: add more primitives supported types!
        _ => panic!(
            "auto-cast from type {ty} isn't implemented in `hs-bindgen`
consider opening an issue https://github.com/yvan-sraka/hs-bindgen"
        ),
    };
    CFFIStub {
        c_ffi_safe_type,
        rust_cast_to_c_ffi,
        hs_c_ffi_type,
    }
}
