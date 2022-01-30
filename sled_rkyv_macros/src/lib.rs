use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

struct KeyParams {
    pub ident: Ident,
    pub case_insensitive: bool,
}

#[proc_macro_derive(Collection, attributes(key))]
pub fn derive_collection(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let key_params = parse_key(&input);
    generate_collection(name, key_params).into()
}

fn parse_key(input: &DeriveInput) -> Option<KeyParams> {
    let mut result = None;
    match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named.iter() {
                    if let Some(key_attr) =
                        field.attrs.iter().find(|attr| attr.path.is_ident("key"))
                    {
                        if result.is_some() {
                            unimplemented!("Only one field can be marked as key");
                        }
                        let meta = key_attr.parse_meta().unwrap();
                        let case_insensitive = match meta {
                            syn::Meta::Path(_) => false,
                            syn::Meta::List(list) => {
                                assert_eq!(list.nested.len(), 1);
                                if let syn::Type::Path(type_path) = &field.ty {
                                    assert!(
                                        type_path.path.is_ident("String"),
                                        "Only String is supported as key"
                                    );
                                } else {
                                    unimplemented!("Only string types can be marked as key");
                                }
                                match list.nested.first() {
                                    Some(syn::NestedMeta::Meta(syn::Meta::Path(path))) => {
                                        if path.is_ident("case_insensitive") {
                                            true
                                        } else {
                                            panic!("Invalid attribute");
                                        }
                                    }
                                    _ => panic!("Invalid attribute"),
                                }
                            }
                            _ => panic!("Only case_insensitive is supported as key attribute"),
                        };

                        result = Some(KeyParams {
                            ident: field.ident.clone().unwrap(),
                            case_insensitive,
                        });
                    }
                }
            }
            _ => {
                unimplemented!("Collection can only be derived for structs with named fields");
            }
        },
        _ => {
            unimplemented!("#[derive(Collection)] is only defined for structs");
        }
    }
    result
}

fn generate_key_serializing(key_params: Option<KeyParams>) -> TokenStream {
    if let Some(key_params) = key_params {
        let ident = &key_params.ident;
        if key_params.case_insensitive {
            quote! {
                type Id = str;

                fn get_id(&self) -> Cow<[u8]> {
                    Cow::Owned(self.#ident.to_lowercase().into_bytes())
                }

                fn build_id(id: &Self::Id) -> Cow<[u8]> {
                    Cow::Owned(id.to_lowercase().into_bytes())
                }
            }
        } else {
            quote! {
                type Id = str;

                fn get_id(&self) -> Cow<[u8]> {
                    Cow::Borrowed(self.#ident.as_bytes())
                }

                fn build_id(id: &Self::Id) -> Cow<[u8]> {
                    Cow::Borrowed(id.as_bytes())
                }
            }
        }
    } else {
        quote! {
            type Id = ();

            fn get_id(&self) -> Cow<[u8]> {
                Cow::Borrowed(b"")
            }

            fn build_id(_id: &Self::Id) -> Cow<[u8]> {
                Cow::Borrowed(b"")
            }
        }
    }
}

fn generate_collection(name: Ident, key_params: Option<KeyParams>) -> TokenStream {
    let key_serializing = generate_key_serializing(key_params);
    quote! {
        const _: () = {
            use std::borrow::Cow;

            sled_rkyv::private::lazy_static! {
                static ref TREE: sled_rkyv::private::sled::Tree =
                    sled_rkyv::private::DB.open_tree(stringify!(#name)).unwrap();
            }
            impl sled_rkyv::Collection for #name {
                #key_serializing

                fn get_tree() -> &'static sled_rkyv::private::sled::Tree {
                    &*TREE
                }
            }
        };
    }
}
