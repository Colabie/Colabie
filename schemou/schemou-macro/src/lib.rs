use proc_macro::TokenStream;

mod derive;

#[proc_macro_derive(Schemou)]
pub fn schemou_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    derive::derive(&ast)
}
