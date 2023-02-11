use crate::{get_type_of, FieldArgs, SilkroadArgs, UsedType};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, Field, Fields, Index, Type};

pub(crate) fn size(ident: &Ident, data: &Data, args: SilkroadArgs) -> TokenStream {
    match *data {
        Data::Struct(ref struct_data) => match &struct_data.fields {
            Fields::Named(named) => {
                let content = named.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    generate_size_for(field, quote!(self.#ident))
                });
                quote_spanned! { ident.span() =>
                    #(#content)+*
                }
            },
            Fields::Unnamed(unnamed) => {
                let content = unnamed.unnamed.iter().enumerate().map(|(i, field)| {
                    let ident = Index::from(i);
                    generate_size_for(field, quote!(self.#ident))
                });
                quote_spanned! { ident.span() =>
                    #(#content)+*
                }
            },
            Fields::Unit => {
                quote!(0)
            },
        },
        Data::Enum(ref enum_data) => {
            let arms = enum_data.variants.iter().map(|variant| {
                let name = &variant.ident;
                match &variant.fields {
                    Fields::Named(named) => {
                        let idents = named
                            .named
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap())
                            .collect::<Vec<&Ident>>();
                        let content = named
                            .named
                            .iter()
                            .zip(&idents)
                            .map(|(field, ident)| generate_size_for(field, quote!(#ident)));

                        quote_spanned! { name.span() =>
                            #ident::#name { #(#idents),* } => {
                                #(#content)+*
                            }
                        }
                    },
                    Fields::Unnamed(unnamed) => {
                        let idents = (0..unnamed.unnamed.len())
                            .map(|i| format_ident!("t{}", i))
                            .collect::<Vec<Ident>>();
                        let content = unnamed
                            .unnamed
                            .iter()
                            .zip(&idents)
                            .map(|(field, ident)| generate_size_for(field, quote!(#ident)));

                        quote_spanned! { name.span() =>
                            #ident::#name(#(#idents),*) => {
                                #(#content)+*
                            }
                        }
                    },
                    Fields::Unit => {
                        quote_spanned! { name.span() =>
                            #ident::#name => 0
                        }
                    },
                }
            });
            let size = args.size.unwrap_or(1);
            quote_spanned! { ident.span() =>
                #size + match &self {
                    #(#arms),*
                }
            }
        },
        Data::Union(_) => {
            quote!(0)
        },
    }
}

fn generate_size_for(field: &Field, ident: TokenStream) -> TokenStream {
    let ty = get_type_of(&field.ty);
    let field_args = FieldArgs::from_attributes(&field.attrs).unwrap();
    match ty {
        UsedType::Primitive => {
            quote_spanned!(field.span() => #ident.byte_size())
        },
        UsedType::String => {
            let size = field_args.size.unwrap_or(1);
            quote_spanned! { field.span() =>
                2 + #ident.len() * #size
            }
        },
        UsedType::Array(_) => {
            quote_spanned!(field.span() => #ident.len())
        },
        UsedType::Collection(inner) => {
            let inner_ty = get_type_of(inner);
            let inner_ts = generate_size_for_inner(inner, &inner_ty, quote!(elem));
            let length_type = field_args.list_type.unwrap_or_else(|| "length".to_string());
            if length_type == "break" || length_type == "has-more" {
                quote_spanned! { field.span() =>
                    1 + #ident.iter().map(|elem| #inner_ts + 1).sum::<usize>()
                }
            } else {
                quote_spanned! { field.span() =>
                    1 + #ident.iter().map(|elem| #inner_ts).sum::<usize>()
                }
            }
        },
        UsedType::Option(inner) => {
            // If we don't have a `when` condition, we need a boolean flag to note if it has content or not.
            // So `is_none()` means no condition, meaning we need another field, and thus size should be 1.
            // If we have a condition, we don't need a field, and thus size should be 0.
            let size = usize::from(field_args.when.is_none());
            let inner_ty = get_type_of(inner);
            let inner_ts = generate_size_for_inner(inner, &inner_ty, quote!(elem));
            quote_spanned! { field.span() =>
                #size + #ident.as_ref().map(|elem| #inner_ts).unwrap_or(0)
            }
        },
        UsedType::Tuple(inner) => {
            let content = (0..inner.len()).map(Index::from).map(|index| {
                quote_spanned! { field.span() =>
                    #ident.#index.byte_size()
                }
            });
            quote_spanned! { field.span() =>
                #(#content)+*
            }
        },
    }
}

fn generate_size_for_inner(ty: &Type, used_type: &UsedType, ident: TokenStream) -> TokenStream {
    match used_type {
        UsedType::Primitive => {
            quote!(#ident.byte_size())
        },
        UsedType::String => {
            quote!(2 + #ident.len())
        },
        UsedType::Array(_) => {
            quote!(#ident.len())
        },
        UsedType::Collection(_) => {
            abort!(ty, "Cannot nest vectors. Create a wrapper struct instead.");
        },
        UsedType::Option(_) => {
            abort!(ty, "Cannot nest options. Create a wrapper struct instead.")
        },
        UsedType::Tuple(inner) => {
            let content = (0..inner.len())
                .map(Index::from)
                .map(|index| quote!(#ident.#index.byte_size()));
            quote!(#(#content)+*)
        },
    }
}
