use proc_macro::TokenStream;
use quote::quote;

pub fn derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    match &ast.data {
        syn::Data::Struct(struct_data) => impl_struct(name, struct_data),
        syn::Data::Enum(enum_data) => impl_enum(name, enum_data),

        _ => panic!(),
    }
}

fn impl_struct(name: &syn::Ident, syn::DataStruct { fields, .. }: &syn::DataStruct) -> TokenStream {
    let is_tuple_struct = matches!(fields, syn::Fields::Unnamed(..));
    let (serialize_fields, deserialize_fields, collection) = (
        // serialization
        fields.iter().enumerate().map(|(field_idx, field)| {
            let field_name = field
                .ident
                .as_ref()
                .map(|i| quote! { #i })
                .unwrap_or_else(|| {
                    let literal = proc_macro2::Literal::usize_unsuffixed(field_idx);
                    quote! { #literal }
                });

            quote! {
                bytes_written += Serde::serialize(&self.#field_name, output);
            }
        }),
        // deserialization
        fields.iter().enumerate().map(|(idx, field)| {
            let ty = &field.ty;
            let field_var_ident = make_ident(&format!("f{idx}"));

            quote! {
                let #field_var_ident = <#ty as Serde>::deserialize(&data.get(offset..)
                    .ok_or(SerdeError::NotEnoughData)?)?;
                offset += #field_var_ident.1;
            }
        }),
        // collection
        fields.iter().enumerate().map(|(idx, field)| {
            let field_var_ident = make_ident(&format!("f{idx}"));

            if is_tuple_struct {
                quote! { #field_var_ident.0 }
            } else {
                let field_name = field.ident.as_ref().unwrap();
                quote! { #field_name: #field_var_ident.0 }
            }
        }),
    );

    let collection = if is_tuple_struct {
        quote! { Self(#(#collection),*) }
    } else {
        quote! { Self{ #(#collection),* } }
    };

    let gen = quote! {
        impl #name {
            #[inline]
            pub fn serialize_buffered(&self) -> Vec<u8> {
                // TODO: preallocate
                let mut data = vec![];
                _ = Serde::serialize(self, &mut data);
                data
            }
        }

        impl Serde for #name {
            fn serialize(&self, output: &mut Vec<u8>) -> usize {
                let mut bytes_written = 0;
                #(#serialize_fields)*
                bytes_written
            }

            fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
                let mut offset = 0;
                #(#deserialize_fields)*

                Ok((
                    #collection, offset
                ))
            }
        }
    };

    gen.into()
}

fn impl_enum(name: &syn::Ident, syn::DataEnum { variants, .. }: &syn::DataEnum) -> TokenStream {
    let serialize = variants.iter().enumerate().map(|(variant_idx, variant)| {
        let (destructure, serialize) = match &variant.fields {
            syn::Fields::Unnamed(unnamed_fields) => {
                let iter = unnamed_fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(field_idx, _)| {
                        proc_macro2::Ident::new(
                            &format!("v{variant_idx}f{field_idx}"),
                            proc_macro2::Span::call_site(),
                        )
                    });

                let destructure = iter
                    .clone()
                    .map(|field_var_ident| quote! { #field_var_ident });

                let serialize = iter
                    .map(|field_var_ident| quote! { bytes_written += Serde::serialize(#field_var_ident, output); });

                (quote! { (#(#destructure)*) }, quote! { #(#serialize)* })
            }

            syn::Fields::Named(named_fields) => {
                let iter = named_fields.named.iter().map(|i| i.ident.as_ref().unwrap());

                let destructure = {
                    let iter = iter.clone();
                    quote! { { #(#iter),* } }
                };

                let serialize = iter
                    .map(|field_var_ident| quote! { bytes_written += Serde::serialize(#field_var_ident, output); });

                (destructure, quote! { #(#serialize)* })
            }

            syn::Fields::Unit => (
                proc_macro2::TokenStream::new(),
                proc_macro2::TokenStream::new(),
            ),
        };

        let variant_name = &variant.ident;

        quote! {
            Self::#variant_name #destructure => {
                bytes_written += (#variant_idx as u32).serialize(output);
                #serialize
            }
        }
    });

    let deserialize = variants.iter().enumerate().map(|(variant_idx, variant)| {
        let deserialize = match &variant.fields {
            syn::Fields::Unnamed(unnamed_fields) => {
                let deserializer = unnamed_fields.unnamed.iter().map(
                    |syn::Field { ty, .. }| quote! {{
                        let (data, inc) = <#ty as Serde>::deserialize(&data.get(offset..).ok_or(SerdeError::NotEnoughData)?)?;
                        offset += inc;
                        data
                    }}
                );

                quote! { (#(#deserializer)*) }
            }

            syn::Fields::Named(named_fields) => {
                let deserializer= named_fields.named.iter().map(
                    |syn::Field { ident, ty, .. }| {
                        let ident = ident.as_ref().unwrap();
                        quote! {
                            #ident: {
                                let (data, inc) = <#ty as Serde>::deserialize(&data.get(offset..).ok_or(SerdeError::NotEnoughData)?)?;
                                offset += inc;
                                data
                            },
                        }
                    }
                );

                quote! { { #(#deserializer)* } }
            }

            syn::Fields::Unit => proc_macro2::TokenStream::new(),
        };

        let variant_idx = variant_idx as u32;
        let variant_name = &variant.ident;

        quote! {
            #variant_idx => Self::#variant_name #deserialize,
        }
    });

    quote! {
        impl #name {
            #[inline]
            pub fn serialize_buffered(&self) -> Vec<u8> {
                // TODO: preallocate
                let mut data = vec![];
                _ = Serde::serialize(self, &mut data);
                data
            }
        }

        impl Serde for #name {
            fn serialize(&self, output: &mut Vec<u8>) -> usize {
                let mut bytes_written = 0;

                match self {
                    #(#serialize)*
                }

                bytes_written
            }

            fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
                let mut offset = 0;
                let (variant_index, shift) = <u32 as Serde>::deserialize(data).unwrap();

                offset += shift;

                Ok((
                    match variant_index {
                        #(#deserialize)*
                        _ => unreachable!()
                    },
                    offset
                ))
            }
        }
    }
    .into()
}

fn make_ident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(string, proc_macro2::Span::call_site())
}
