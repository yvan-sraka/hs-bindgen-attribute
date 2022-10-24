use crate::haskell;
use antlion::Sandbox;
use hs_bindgen_traits::HsType;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

// FIXME: I plan to add this in next release :)
const UNSUPPORTED_RETURN_TYPE: &str =
    "`hs-bindgen` currently only support Haskell function signature that end by returning `IO ()`";

/// Generate extra Rust code that wrap our exposed function
pub(crate) fn generate(
    sig: Option<haskell::Signature>,
    item_fn: syn::ItemFn,
) -> (haskell::Signature, TokenStream) {
    let rust_fn = format_ident!("{}", item_fn.sig.ident.to_string());
    let mut sig = match sig {
        Some(sig) => sig,
        None => haskell::Signature {
            fn_name: rust_fn.to_string(),
            fn_type: infer_hs_type(item_fn),
        },
    };

    // FIXME: ensure that Haskell signature end by `IO ()` type ...
    let x: HsType = sig.fn_type.pop().expect(UNSUPPORTED_RETURN_TYPE);
    assert!(x.to_string() == "IO ()", "{UNSUPPORTED_RETURN_TYPE}");

    let mut c_fn_args = quote! {};
    let mut rust_fn_values = quote! {};
    for (i, hs_c_ffi_type) in sig.fn_type.iter().enumerate() {
        let arg = format_ident!("__{i}");
        let c_ffi_safe_type = hs_c_ffi_type.quote();
        c_fn_args.extend(quote! { #arg: #c_ffi_safe_type, });
        rust_fn_values.extend(quote! { traits::ReprC::from(#arg), });
    }

    let c_fn = format_ident!("__c_{}", sig.fn_name);
    let extern_c_wrapper = quote! {
        #[no_mangle] // Mangling randomize symbols
        extern "C" fn #c_fn(#c_fn_args) {
            // FIXME: this is a trick to currently not allow function that
            // return argument, indeed this should be fixed ...
            #rust_fn(#rust_fn_values)
        }
    };

    // FIXME: again same hack, to force function to return `IO ()` ...
    sig.fn_type.push(HsType::IO(Box::new(HsType::Empty)));

    // DEBUG: println!("{extern_c_wrapper}");
    (sig, extern_c_wrapper.into())
}

fn infer_hs_type(item_fn: syn::ItemFn) -> Vec<HsType> {
    let mut fn_type = vec![];
    for input in &item_fn.sig.inputs {
        let ty = match input {
            syn::FnArg::Typed(p) => &p.ty,
            _ => panic!("functions using `self` are not supported by `hs-bindgen`"),
        };
        // FIXME: this handy feature slow down a lot compilation, so either
        // `antlion` should smarter cache sandbox or either this feature should
        // disabled by default under a `infer_hs_type` feature
        let sandbox = Sandbox::new()
            .unwrap()
            .deps(&["hs-bindgen-traits@0.5"])
            .unwrap();
        let hs_c_ffi_type: HsType = sandbox
            .eval(quote! {
                <#ty as hs_bindgen_traits::ReprHs>::into()
            })
            .unwrap_or_else(|_| {
                panic!(
                    "type `{}` doesn't implement `ReprHs` trait
consider opening an issue https://github.com/yvan-sraka/hs-bindgen-traits

n.b. if you trying to use a custom defined type, you need to specify the
Haskell type signature of your binding: #[hs_bindgen(HASKELL TYPE SIGNATURE)]",
                    quote! { #ty }
                )
            });
        fn_type.push(hs_c_ffi_type);
    }

    // FIXME: this is required by e.g. usage of Haskell `newCString` ...
    fn_type.push(HsType::IO(Box::new(HsType::Empty)));
    // ... rather than `HsType::Empty` should contain Rust return type!

    fn_type
}
