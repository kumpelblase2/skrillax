use crate::deserialize::deserialize;
use crate::serialize::serialize;
use crate::size::size;
use darling::{FromAttributes, FromDeriveInput};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Expr, GenericArgument, PathArguments, Type};

mod deserialize;
mod serialize;
mod size;

pub(crate) const DEFAULT_LIST_TYPE: &str = "length";

#[derive(FromAttributes)]
#[darling(attributes(silkroad))]
pub(crate) struct FieldArgs {
    list_type: Option<String>,
    size: Option<usize>,
    value: Option<usize>,
    when: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(silkroad))]
pub(crate) struct SilkroadArgs {
    size: Option<usize>,
}

#[proc_macro_error]
#[proc_macro_derive(Serialize, attributes(silkroad))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let args = SilkroadArgs::from_derive_input(&input).unwrap();
    let DeriveInput { ident, data, .. } = input;

    let output = serialize(&ident, &data, args);

    let output = quote! {
        impl Serialize for #ident {
            fn write_to(&self, mut writer: &mut bytes::BytesMut) {
                #output
            }
        }

        impl From<#ident> for bytes::Bytes {
            fn from(packet: #ident) -> bytes::Bytes {
                let mut buffer = bytes::BytesMut::with_capacity(packet.byte_size());
                packet.write_to(&mut buffer);
                buffer.freeze()
            }
        }
    };
    output.into()
}

#[proc_macro_error]
#[proc_macro_derive(Deserialize, attributes(silkroad))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let args = SilkroadArgs::from_derive_input(&input).unwrap();
    let DeriveInput { ident, data, .. } = input;
    let output = deserialize(&ident, &data, args);
    let output = quote! {
        impl Deserialize for #ident {
            fn read_from<T: std::io::Read + byteorder::ReadBytesExt>(mut reader: &mut T) -> Result<Self, SerializationError> {
                #output
            }
        }

        impl TryFrom<bytes::Bytes> for #ident {
            type Error = SerializationError;

            fn try_from(data: bytes::Bytes) -> Result<Self, Self::Error> {
                use bytes::Buf;
                let mut data_reader = data.reader();
                #ident::read_from(&mut data_reader)
            }
        }
    };
    output.into()
}

#[proc_macro_error]
#[proc_macro_derive(ByteSize, attributes(silkroad))]
pub fn derive_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let args = SilkroadArgs::from_derive_input(&input).unwrap();
    let DeriveInput { ident, data, .. } = input;
    let output = size(&ident, &data, args);
    let output = quote! {
        impl ByteSize for #ident {
            fn byte_size(&self) -> usize {
                #output
            }
        }
    };
    output.into()
}

#[derive(Debug)]
pub(crate) enum UsedType<'a> {
    Primitive,
    String,
    Array(&'a Expr),
    Collection(&'a Type),
    Option(&'a Type),
    Tuple(Vec<&'a Type>),
}

pub(crate) fn get_type_of(ty: &Type) -> UsedType {
    match ty {
        Type::Array(arr) => UsedType::Array(&arr.len),
        Type::Reference(_) => abort!(ty, "References are not supported for (de)serialization."),
        Type::Tuple(tuple) => UsedType::Tuple(tuple.elems.iter().collect()),
        Type::Path(path) => {
            let full_name = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<String>>()
                .join("::");

            if full_name == "String" || full_name == "string::String" || full_name == "std::string::String" {
                return UsedType::String;
            } else if full_name == "Vec" {
                match path.path.segments.last().unwrap().arguments {
                    PathArguments::None => abort!(ty, "Missing generic parameters for collection type."),
                    PathArguments::Parenthesized(_) => abort!(ty, "Cannot use parenthesized types."),
                    PathArguments::AngleBracketed(ref args) => {
                        let ty = args
                            .args
                            .iter()
                            .find_map(|arg| match arg {
                                GenericArgument::Type(ty) => Some(ty),
                                _ => None,
                            })
                            .unwrap();
                        return UsedType::Collection(ty);
                    },
                }
            } else if full_name == "Option" {
                match path.path.segments.last().unwrap().arguments {
                    PathArguments::None => abort!(ty, "Missing generic parameters for option type."),
                    PathArguments::Parenthesized(_) => abort!(ty, "Cannot use parenthesized types."),
                    PathArguments::AngleBracketed(ref args) => {
                        let ty = args
                            .args
                            .iter()
                            .find_map(|arg| match arg {
                                GenericArgument::Type(ty) => Some(ty),
                                _ => None,
                            })
                            .unwrap();
                        return UsedType::Option(ty);
                    },
                }
            }

            UsedType::Primitive
        },
        _ => abort!(ty, "Encountered unknown syn type."),
    }
}

fn get_variant_value<T: Spanned + ToTokens>(source: &T, value: usize, size: usize) -> syn::Expr {
    let ty = match size {
        1 => "u8",
        2 => "u16",
        4 => "u32",
        8 => "u64",
        _ => abort!(source, "Unknown size"),
    };
    syn::parse_str(&format!("{}{}", value, ty)).unwrap()
}
