use crate::{get_type_of, get_variant_value, FieldArgs, SilkroadArgs, UsedType};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, Expr, Field, Fields, Type};

pub(crate) fn deserialize(ident: &Ident, data: &Data, args: SilkroadArgs) -> TokenStream {
    match *data {
        Data::Struct(ref struct_data) => match &struct_data.fields {
            Fields::Named(named) => {
                let idents = named
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect::<Vec<&Ident>>();
                let content = named
                    .named
                    .iter()
                    .map(|field| generate_reader_for(field, field.ident.as_ref().unwrap()));
                quote_spanned! {ident.span()=>
                    #(#content)*
                    Ok(#ident { #(#idents),* })
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
                    .map(|(field, ident)| generate_reader_for(field, ident));
                quote_spanned! { ident.span() =>
                    #(#content)*
                    Ok(#ident(#(#idents),*))
                }
            },
            Fields::Unit => {
                quote_spanned! { ident.span() =>
                    Ok(#ident)
                }
            },
        },
        Data::Enum(ref enum_data) => {
            let enum_size = args.size.unwrap_or(1);
            let arms = enum_data.variants.iter().map(|variant| {
                let field_args = FieldArgs::from_attributes(&variant.attrs).unwrap();
                let value = get_variant_value(
                    &variant.ident,
                    field_args.value.expect("Missing value for variant."),
                    enum_size,
                );
                let variant_ident = &variant.ident;

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
                            .map(|field| generate_reader_for(field, field.ident.as_ref().unwrap()));
                        quote_spanned! { variant_ident.span() =>
                            #value => {
                                #(#content)*
                                Ok(#ident::#variant_ident { #(#idents),* })
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
                            .map(|(field, ident)| generate_reader_for(field, ident));
                        quote_spanned! { variant_ident.span() =>
                            #value => {
                                #(#content)*
                                Ok(#ident::#variant_ident(#(#idents),*))
                            }
                        }
                    },
                    Fields::Unit => {
                        quote_spanned! { variant_ident.span() =>
                            #value => Ok(#ident::#variant_ident)
                        }
                    },
                }
            });

            let variant_string = format!("{}", ident);
            let size = args.size.unwrap_or(1);
            let reader = match size {
                1 => quote_spanned!(ident.span() => u8::read_from(reader)?),
                2 => quote_spanned!(ident.span() => u16::read_from(reader)?),
                4 => quote_spanned!(ident.span() => u32::read_from(reader)?),
                8 => quote_spanned!(ident.span() => u64::read_from(reader)?),
                _ => abort!(ident, "Invalid size"),
            };
            quote_spanned! { ident.span() =>
                match #reader {
                    #(#arms),*,
                    unknown => Err(silkroad_serde::SerializationError::UnknownVariation(unknown as usize, #variant_string)),
                }
            }
        },
        _ => abort!(ident, "Unions are not supported."),
    }
}

fn generate_reader_for(field: &Field, ident: &Ident) -> TokenStream {
    let ty = get_type_of(&field.ty);
    let type_name = &field.ty;
    let args = FieldArgs::from_attributes(&field.attrs).unwrap();
    match ty {
        UsedType::Primitive => {
            quote_spanned! { field.span() =>
                let #ident = #type_name::read_from(reader)?;
            }
        },
        UsedType::String => {
            let content = match args.size.unwrap_or(1) {
                1 => quote! {
                    for _ in 0..len {
                        bytes.push(u8::read_from(reader)?);
                    }
                    let #ident = String::from_utf8(bytes)?;
                },
                2 => quote! {
                    for _ in 0..len {
                        bytes.push(u16::read_from(reader)?);
                    }
                    let #ident = String::from_utf16(&bytes)?;
                },
                _ => abort!(field, "Unknown String size"),
            };

            quote_spanned! { field.span() =>
                let len = u16::read_from(reader)?;
                let mut bytes = Vec::with_capacity(len.into());
                #content
            }
        },
        UsedType::Array(len) => {
            quote_spanned! { field.span() =>
                let mut bytes = [0u8; #len];
                reader.read_exact(&mut bytes)?;
                let #ident = bytes;
            }
        },
        UsedType::Collection(inner) => {
            let inner_ty = get_type_of(inner);
            let inner = generate_reader_for_inner(ident, inner, &inner_ty);
            quote_spanned! { field.span() =>
                let size = u8::read_from(reader)?;
                let mut items = Vec::with_capacity(size.into());
                for _ in 0..size {
                    #inner
                    items.push(#ident);
                }
                let #ident = items;
            }
        },
        UsedType::Option(inner) => {
            let inner_ty = get_type_of(inner);
            let inner_ts = generate_reader_for_inner(ident, inner, &inner_ty);
            match args.when {
                Some(condition) => {
                    if let Ok(condition) = syn::parse_str::<Expr>(&condition) {
                        quote_spanned! { field.span() =>
                            let #ident = if #condition {
                                #inner_ts
                                Some(#ident)
                            } else {
                                None
                            };
                        }
                    } else {
                        abort!(field, "Condition could not be parsed");
                    }
                },
                None => {
                    quote_spanned! { field.span() =>
                        let some = u8::read_from(reader)?;
                        let #ident = if some == 1 {
                            #inner_ts
                            Some(#ident)
                        } else {
                            None
                        };
                    }
                },
            }
        },
        UsedType::Tuple(inner) => {
            let idents = (0..inner.len())
                .map(|i| format_ident!("t{}", i))
                .collect::<Vec<Ident>>();
            let content = inner.iter().zip(&idents).map(|(ty, ident)| {
                let inner_ty = get_type_of(ty);
                generate_reader_for_inner(ident, ty, &inner_ty)
            });
            quote_spanned! { field.span() =>
                #(#content)*
                let #ident = (#(#idents),*);
            }
        },
    }
}

fn generate_reader_for_inner(ident: &Ident, type_name: &Type, ty: &UsedType) -> TokenStream {
    match ty {
        UsedType::Primitive => {
            quote_spanned! { ident.span() =>
                let #ident = #type_name::read_from(reader)?;
            }
        },
        UsedType::String => {
            quote_spanned! { ident.span() =>
                let len = u16::read_from(reader)?;
                let mut bytes = Vec::with_capacity(len.into());
                for _ in 0..len {
                    bytes.push(u8::read_from(reader)?);
                }
                let #ident = String::from_utf8(bytes)?;
            }
        },
        UsedType::Array(len) => {
            quote_spanned! { ident.span() =>
                let mut bytes = [0u8; #len];
                reader.read_exact(bytes)?;
                let #ident = bytes;
            }
        },
        UsedType::Collection(inner) => {
            quote_spanned! { ident.span() =>
                let size = u8::read_from(reader)?;
                let mut items = Vec::with_capacity(size.into());
                for _ in 0..size {
                    items.push(#inner::read_from(reader)?);
                }
                let #ident = items;
            }
        },
        UsedType::Option(inner) => {
            quote_spanned! { ident.span() =>
                let some = u8::read_from(reader)?;
                let #ident = if some == 1 {
                    Some(#inner::read_from(reader)?)
                } else {
                    None
                };
            }
        },
        UsedType::Tuple(inner) => {
            let content = inner.iter().map(|ty| quote!(#ty::read_from(reader)?));
            quote_spanned! { ident.span() =>
                let #ident = (#(#content),*);
            }
        },
    }
}
