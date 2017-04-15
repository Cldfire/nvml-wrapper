#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::MetaItem::*;
use syn::Lit::*;
use syn::Body::*;
use syn::NestedMetaItem::*;

// TODO: Tests.
// TODO: Try to turn this abhorrent code into something you can look at without losing your eyes.

// Use it like this
//
// #[derive(EnumWrapper)]
// #[wrap(c_enum = "nvmlSomeEnum_t")]
// #[wrap(has_count = "COUNT_VARIANT")] (optional)
// pub enum SomeEnum {
//     #[wrap(c_variant = NVML_SOME_VARIANT)]
//     SomeVariant,
//     #[wrap(c_variant = NVML_OTHER_VARIANT)]
//     SomeOtherVariant,
// }

struct VariantInfo {
    rust_name: syn::Ident,
    rust_variant: syn::Ident,
    c_name: syn::Ident,
    c_variant: syn::Ident,
}

impl VariantInfo {
    fn from(variant: syn::Variant, c_name: syn::Ident, rust_name: syn::Ident) -> Self {
        let c_variant: syn::Ident = variant_attr_val_for_str("c_variant", &variant).into();

        VariantInfo {
            rust_name: rust_name,
            rust_variant: variant.ident,
            c_name: c_name,
            c_variant: c_variant,
        }
    }

    fn tokens_for_eq_c(&self) -> Tokens {
        let ref rust_name = self.rust_name;
        let ref rust_variant = self.rust_variant;
        let ref c_name = self.c_name;
        let ref c_variant = self.c_variant;

        quote! {
            #rust_name::#rust_variant => #c_name::#c_variant,
        }
    }

    fn tokens_for_from_c(&self) -> Tokens {
        let ref rust_name = self.rust_name;
        let ref rust_variant = self.rust_variant;
        let ref c_name = self.c_name;
        let ref c_variant = self.c_variant;

        quote! {
            #c_name::#c_variant => #rust_name::#rust_variant,
        }
    }

    fn tokens_for_try_from_c(&self) -> Tokens {
        let ref rust_name = self.rust_name;
        let ref rust_variant = self.rust_variant;
        let ref c_name = self.c_name;
        let ref c_variant = self.c_variant;

        quote! {
            #c_name::#c_variant => Ok(#rust_name::#rust_variant),
        }
    }
}

#[proc_macro_derive(EnumWrapper, attributes(wrap))]
pub fn enum_wrapper(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).expect("Could not parse derive input");

    let expanded = wrap_enum(ast);

    expanded.parse().expect("Could not parse expanded output")
}

fn wrap_enum(ast: syn::DeriveInput) -> Tokens {
    let rust_name = &ast.ident;
    let c_name: syn::Ident = attr_val_for_str("c_enum", &ast).unwrap().into();
    let count_variant = attr_val_for_str("has_count", &ast);

    match ast.body {
        Enum(variant_vec) => {
            let info_vec: Vec<VariantInfo> = variant_vec.iter().map(|v| {
                VariantInfo::from(v.clone(), c_name.clone(), rust_name.clone())
            }).collect();
            
            if let Some(v) = count_variant {
                gen_impl(&info_vec[..], Some(v.into()))
            } else {
                gen_impl(&info_vec[..], None)
            }
        },
        Struct(_) => panic!("This derive macro does not support structs"),
    }

}

fn gen_impl(variant_slice: &[VariantInfo], count_variant: Option<syn::Ident>) -> Tokens {
    let ref c_name = variant_slice[0].c_name;
    let ref rust_name = variant_slice[0].rust_name;

    let for_arms: Vec<Tokens> = variant_slice.iter().map(|v| {
        v.tokens_for_eq_c()
    }).collect();

    let from_arms: Vec<Tokens> = variant_slice.iter().map(|v| {
        v.tokens_for_from_c()
    }).collect();

    let try_from_arms: Vec<Tokens> = variant_slice.iter().map(|v| {
        v.tokens_for_try_from_c()
    }).collect();

    // TODO: Add error docs to `try_from`
    if let Some(v) = count_variant {
        quote! {
            impl #rust_name {
                /// Returns the C enum variant equivalent for the given Rust enum variant.
                pub fn into_c(&self) -> #c_name {
                    match *self {
                        #(#for_arms)*
                    }
                }

                /// Waiting for `TryFrom` to be stable. In the meantime, we do this.
                pub fn try_from(enum_: #c_name) -> Result<Self> {
                    match enum_ {
                        #(#try_from_arms)*
                        #c_name::#v => Err(Error::from_kind(ErrorKind::UnexpectedVariant)),
                    }
                }
            }
        }
    } else {
        quote! {
            impl #rust_name {
                /// Returns the C enum variant equivalent for the given Rust enum variant.
                pub fn into_c(&self) -> #c_name {
                    match *self {
                        #(#for_arms)*
                    }
                }
            }

            impl From<#c_name> for #rust_name {
                fn from(enum_: #c_name) -> Self {
                    match enum_ {
                        #(#from_arms)*
                    }
                }
            }
        }
    }
}

// TODO: This... is so bad. 
 fn attr_val_for_str<S: AsRef<str>>(string: S, ast: &syn::DeriveInput) -> Option<String> {
    let mut return_string: Option<String> = None;
    // Iterate through attributes on this variant, match on the MetaItem
    ast.attrs.iter().find(|ref a| match a.value {
        // If this value is a List...
        List(ref ident, ref nested_items_vec) => {
            let mut real_return_val = false;
            // If the ident matches our derive's prefix...
            if ident == "wrap" {
                // Iterate through nested attributes in this attribute and match on NestedMetaItem...
                let item = nested_items_vec.iter().find(|ref i| match i {
                    // If it's another MetaItem
                    &&&MetaItem(ref item) => match item {
                        // If it's a name value pair
                        &NameValue(ref ident, ref lit) => {
                            let mut return_val = false;
                            // If the name matches what was passed in for us to look for
                            if ident == string.as_ref() {
                                // Match on the value paired with the name
                                return_string = match lit {
                                    // If it's a string, return it. Then go beg for mercy after
                                    // having read through this code.
                                    &Str(ref the_value, _) => Some(the_value.to_string()),
                                    _ => panic!("Attribute value was not a string")
                                };
                                return_val = true;
                            }
                            return_val
                        },
                        _ => panic!("Attribute was was not a namevalue"),
                    },
                    _ => false,
                });

                if let Some(_) = item {
                    real_return_val = true;
                }
            }
            real_return_val
        },
        _ => false,
    });

    return_string
}

// TODO: And this... is even worse.
fn variant_attr_val_for_str<S: AsRef<str>>(string: S, variant: &syn::Variant) -> String {
    let mut return_string = "this_is_not_to_be_returned".to_string();
    variant.attrs.iter().find(|ref a| match a.value {
        List(ref ident, ref nested_items_vec) => {
            let mut real_return_val = false;
            if ident == "wrap" {
                let item = nested_items_vec.iter().find(|ref i| match i {
                    &&&MetaItem(ref item) => match item {
                        &NameValue(ref ident, ref lit) => {
                            let mut return_val = false;
                            if ident == string.as_ref() {
                                return_string = match lit {
                                    &Str(ref the_value, _) => the_value.to_string(),
                                    _ => panic!("Attribute value was not a string")
                                };
                                return_val = true;
                            }
                            return_val
                        },
                        _ => panic!("Attribute was was not a namevalue"),
                    },
                    _ => false,
                });

                if let Some(_) = item {
                    real_return_val = true;
                }
            }
            real_return_val
        },
        _ => false,
    });

    if return_string != "this_is_not_supposed_to_be_returned" {
        return_string
    } else {
        panic!("Could not find attribute for {:?}", string.as_ref())
    }
}
