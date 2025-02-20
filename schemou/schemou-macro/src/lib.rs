use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Schemou)]
pub fn schemou_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    impl_schemou(&ast)
}

fn impl_schemou(ast: &syn::DeriveInput) -> TokenStream {
    let (serialize_fields, deserialize_fields, collection) = match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => (
            // serialization
            fields.iter().map(|field| {
                let field_name = field
                    .ident
                    .as_ref()
                    .expect("Tuple structs are not supported.");

                quote! {
                    data.extend(Serde::serialize(&self.#field_name));
                }
            }),
            // deserialization
            fields.iter().enumerate().map(|(idx, field)| {
                let ty = &field.ty;

                let field_var_ident = proc_macro2::Ident::new(
                    &format!("field_{idx}"),
                    proc_macro2::Span::call_site(),
                );

                quote! {
                    let #field_var_ident = <#ty as Serde>::deserialize(&data.get(offset..)
                        .ok_or(SerdeError::NotEnoughData)?)?;
                    offset += #field_var_ident.1;
                }
            }),
            // collection
            fields.iter().enumerate().map(|(idx, field)| {
                let field_name = field
                    .ident
                    .as_ref()
                    .expect("Tuple structs are not supported.");

                let field_var_ident = proc_macro2::Ident::new(
                    &format!("field_{idx}"),
                    proc_macro2::Span::call_site(),
                );

                quote! { #field_name: #field_var_ident.0 }
            }),
        ),

        _ => panic!("Schemou can only be derived for structs."),
    };

    let name = &ast.ident;
    let gen = quote! {
        impl Serde for #name {
            fn serialize(&self) -> Vec<u8>{
                let mut data = vec![];
                #(#serialize_fields)*
                data
            }

            fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
                let mut offset = 0;
                #(#deserialize_fields)*

                Ok((
                    Self { #(#collection),* }, offset
                ))
            }
        }
    };

    gen.into()
}
