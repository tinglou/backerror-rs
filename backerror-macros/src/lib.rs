#![allow(unused_imports, dead_code)]
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, Item, ItemEnum, ItemStruct, Meta, Path, Token,
};

#[cfg(not(any(not(feature = "release_off"), debug_assertions)))]
#[proc_macro_attribute]
pub fn backerror(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[cfg(any(not(feature = "release_off"), debug_assertions))]
#[proc_macro_attribute]
pub fn backerror(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input2 = input.clone();
    let item = parse_macro_input!(input2 as Item);

    match item {
        Item::Enum(item_enum) => backerror_enum(item_enum, input),
        Item::Struct(item_struct) => backerror_struct(item_struct, input),
        _ => input,
    }
}

/// enum error
///
/// ```ignore
/// #[backerror]
/// #[derive(Debug, Error)]
/// pub enum MyError {
///     #[error("{0}")]
///     IoError(#[from] std::io::Error),
/// }
/// ```
fn backerror_enum(mut item_enum: ItemEnum, input: TokenStream) -> TokenStream {
    // check whether the enum derives thiserror::Error
    if !check_derive_thiserror(&item_enum.attrs) {
        return input;
    }

    let mut error_types = Vec::new();

    for variant in item_enum.variants.iter_mut() {
        let fields = &mut variant.fields;
        enhance_fields(fields, &mut error_types);
    }

    if let Ok(impls) = generate_from_impl(&item_enum.ident, &error_types) {
        let ret = quote! {
            #item_enum
            #impls
        };

        ret.into()
    } else {
        input
    }
}

/// transparent struct
///
/// ```ignore
/// #[backerror]
/// #[derive(Debug, Error)]
/// #[error(transparent)]
/// pub struct MyError(#[from] std::io::Error);
/// ```
fn backerror_struct(mut item_struct: ItemStruct, input: TokenStream) -> TokenStream {
    // check whether the struct derives thiserror::Error
    if !check_derive_thiserror(&item_struct.attrs) || !check_transparent_struct(&item_struct.attrs)
    {
        return input;
    }

    let mut error_types = Vec::new();

    let fields = &mut item_struct.fields;
    enhance_fields(fields, &mut error_types);

    if let Ok(impls) = generate_from_impl(&item_struct.ident, &error_types) {
        let ret = quote! {
            #item_struct
            #impls
        };

        ret.into()
    } else {
        input
    }
}

fn generate_from_impl(
    ident: &Ident,
    error_types: &Vec<String>,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    if error_types.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "no attribute found",
        ));
    }

    let mut impls = Vec::new();
    for e in error_types {
        let from_ty: Path = syn::parse_str(e)?;
        let block = quote! {
            impl From<#from_ty> for #ident {
                #[track_caller]
                fn from(e: #from_ty) -> Self {
                    #ident::from(backerror::LocatedError::from(e))
                }
            }
        };
        impls.push(block);
    }

    Ok(quote! {
        #(#impls)*
    })
}

/// enhance fiels from `#[from] T` to `#[from] backerror::LocatedError<T>`
fn enhance_fields(fields: &mut syn::Fields, errors: &mut Vec<String>) {
    match fields {
        syn::Fields::Unnamed(fs) => {
            for field in fs.unnamed.iter_mut() {
                if check_attr_from(&field.attrs) {
                    let orig_ty = field.ty.clone().into_token_stream().to_string();
                    let ty = format!("backerror::LocatedError<{}>", orig_ty);
                    if let Ok(new_type) = syn::parse_str(&ty) {
                        errors.push(orig_ty);
                        field.ty = new_type;
                    } else {
                        println!("failed to parse {}", ty);
                    }
                } else {
                    // println!("transparent struct field without #[from]");
                }
            }
        }
        syn::Fields::Unit | syn::Fields::Named(_) => {
            // do nothing
        }
    }
}

/// check `#[derive(Error)]`
fn check_derive_thiserror(attrs: &Vec<syn::Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("derive") {
            if let Ok(nested) =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            {
                for meta in nested {
                    match meta {
                        Meta::Path(path) => {
                            // #[derive(Error)]
                            if path.is_ident("Error") {
                                return true;
                            }
                            // #[derive(thiserror::Error)]
                            let path = path.into_token_stream().to_string();
                            if path.contains("thiserror") && path.contains("Error") {
                                return true;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
    return false;
}

/// check `#[error(transparent)]`
fn check_transparent_struct(attrs: &Vec<syn::Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("error") {
            if let Ok(nested) =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            {
                for meta in nested {
                    match meta {
                        Meta::Path(path) => {
                            // #[error(transparent)]
                            if path.is_ident("transparent") {
                                return true;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
    return false;
}

/// check `#[from]`
fn check_attr_from(attrs: &Vec<syn::Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("from") {
            return true;
        }
    }
    return false;
}
