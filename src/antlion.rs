use crate::haskell;
use hs_bindgen_traits::HsType;

/// Use Rust type inference (inside a `antlion` sandbox) to deduce targeted
/// Haskell type signature that match a given `TokenStream` of a Rust `fn`
pub(crate) trait Eval<T> {
    fn from(_: T) -> Self;
}

impl Eval<&syn::ItemFn> for haskell::Signature {
    #[cfg(not(feature = "antlion"))]
    fn from(_: &syn::ItemFn) -> Self {
        unreachable!()
    }
    #[cfg(feature = "antlion")]
    fn from(item_fn: &syn::ItemFn) -> Self {
        let fn_name = item_fn.sig.ident.to_string();
        let mut fn_type = vec![];
        for arg in &item_fn.sig.inputs {
            fn_type.push(<HsType as Eval<&syn::Type>>::from(match arg {
                syn::FnArg::Typed(p) => &p.ty,
                _ => panic!("functions using `self` are not supported by `hs-bindgen`"),
            }));
        }
        fn_type.push(HsType::IO(Box::new(match &item_fn.sig.output {
            syn::ReturnType::Type(_, p) => <HsType as Eval<&syn::Type>>::from(p),
            _ => HsType::Empty,
        })));
        haskell::Signature { fn_name, fn_type }
    }
}

#[cfg(feature = "antlion")]
impl Eval<&syn::Type> for HsType {
    fn from(ty: &syn::Type) -> HsType {
        use antlion::Sandbox;
        use quote::quote;
        // FIXME: This should rather be instantiate in a `lazy_static!` block ...
        let sandbox = Sandbox::new("hs-bindgen")
            .unwrap()
            .deps(&["hs-bindgen-traits@0.5"])
            .unwrap();
        sandbox
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
            })
    }
}
