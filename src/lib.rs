use proc_macro::TokenStream;

mod toml;
mod rust;
mod haskell;

#[proc_macro_attribute]
pub fn hs_bindgen(attrs: TokenStream, input: TokenStream) -> TokenStream {
    // FIXME: macro attributes would be used to customize, exposed C FFI
    // function name, or the path haskell module where binding are generated
    let mut output = input.clone();
    output.extend(rust::generate(syn::parse(input).unwrap()));
    output
}
