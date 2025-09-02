extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

fn expand(input: DeriveInput) -> proc_macro2::TokenStream {
    let ty_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref fields_named) => {
                let acc_fns = fields_named.named.iter().map(|f| {
                    let fname: &Ident = f.ident.as_ref().unwrap();
                    let acc_fn = format_ident!("acc_{}", fname);
                    let fty = &f.ty;
                    quote! {
                        /// Accessor to the `#fname` field.
                        pub const fn #acc_fn() -> pathmod::Accessor<#ty_ident #ty_generics, #fty> {
                            let off = core::mem::offset_of!(#ty_ident #ty_generics, #fname) as isize;
                            // SAFETY: `off` is computed from the field offset within the same allocation.
                            unsafe { pathmod::Accessor::<#ty_ident #ty_generics, #fty>::from_offset(off) }
                        }
                    }
                });

                let with_fns = fields_named.named.iter().map(|f| {
                    let fname: &Ident = f.ident.as_ref().unwrap();
                    let with_fn = format_ident!("with_{}", fname);
                    let fty = &f.ty;
                    quote! {
                        /// Return a new value with `#fname` replaced by `new_val`.
                        ///
                        /// This consumes `self` and reconstructs `Self` without cloning
                        /// any other fields (they are moved). This is the building block
                        /// for minimal-clone (actually zero-clone) reconstruction up the path.
                        pub fn #with_fn(mut self, new_val: #fty) -> Self {
                            self.#fname = new_val;
                            self
                        }
                    }
                });

                quote! {
                    impl #impl_generics #ty_ident #ty_generics #where_clause {
                        #(#acc_fns)*
                        #(#with_fns)*
                    }
                }
            }
            Fields::Unnamed(ref fields_unnamed) => {
                let acc_fns = fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                    let acc_fn = format_ident!("acc_{}", i);
                    let fty = &f.ty;
                    let index = syn::Index::from(i);
                    quote! {
                        /// Accessor to the tuple field at index #i.
                        pub const fn #acc_fn() -> pathmod::Accessor<#ty_ident #ty_generics, #fty> {
                            let off = core::mem::offset_of!(#ty_ident #ty_generics, #index) as isize;
                            // SAFETY: `off` is computed from the field offset within the same allocation.
                            unsafe { pathmod::Accessor::<#ty_ident #ty_generics, #fty>::from_offset(off) }
                        }
                    }
                });
                let with_fns = fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                    let with_fn = format_ident!("with_{}", i);
                    let fty = &f.ty;
                    let index = syn::Index::from(i);
                    quote! {
                        /// Return a new value with tuple field at index #i replaced by `new_val`.
                        ///
                        /// Consumes `self` and reconstructs `Self` without cloning other fields.
                        pub fn #with_fn(mut self, new_val: #fty) -> Self {
                            self.#index = new_val;
                            self
                        }
                    }
                });
                quote! {
                    impl #impl_generics #ty_ident #ty_generics #where_clause {
                        #(#acc_fns)*
                        #(#with_fns)*
                    }
                }
            }
            Fields::Unit => {
                let msg = "#[derive(Accessor)] does not support unit structs";
                quote! { compile_error!(#msg); }
            }
        },
        _ => {
            let msg = "#[derive(Accessor)] can only be used on structs";
            quote! { compile_error!(#msg); }
        }
    }
}

#[proc_macro_derive(Accessor)]
pub fn accessor_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let ts = expand(input);
    TokenStream::from(ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn expands_named_struct() {
        let di: DeriveInput = parse_quote! {
            struct S { a: i32, b: i64 }
        };
        let out = expand(di);
        let s = out.to_string();
        assert!(s.contains("acc_a"));
        assert!(s.contains("acc_b"));
    }

    #[test]
    fn expands_tuple_struct() {
        let di: DeriveInput = parse_quote! {
            struct P(i32, i64);
        };
        let out = expand(di);
        let s = out.to_string();
        assert!(s.contains("acc_0"));
        assert!(s.contains("acc_1"));
    }

    #[test]
    fn errors_on_unit_struct() {
        let di: DeriveInput = parse_quote! { struct U; };
        let out = expand(di);
        let s = out.to_string();
        assert!(s.contains("compile_error") && s.contains("does not support unit structs"));
    }

    #[test]
    fn errors_on_enum() {
        let di: DeriveInput = parse_quote! { enum E { A } };
        let out = expand(di);
        let s = out.to_string();
        assert!(s.contains("compile_error") && s.contains("only be used on structs"));
    }
}
