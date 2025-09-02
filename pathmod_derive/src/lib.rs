extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(Accessor)]
pub fn accessor_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let ty_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let gen = match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref fields_named) => {
                let fns = fields_named.named.iter().map(|f| {
                    let fname: &Ident = f.ident.as_ref().unwrap();
                    let acc_fn = format_ident!("acc_{}", fname);
                    let fty = &f.ty;
                    quote! {
                        /// Accessor to the `#fname` field.
                        pub const fn #acc_fn() -> pathmod_core::Accessor<#ty_ident #ty_generics, #fty> {
                            let off = core::mem::offset_of!(#ty_ident #ty_generics, #fname) as isize;
                            // SAFETY: `off` is computed from the field offset within the same allocation.
                            unsafe { pathmod_core::Accessor::<#ty_ident #ty_generics, #fty>::from_offset(off) }
                        }
                    }
                });

                quote! {
                    impl #impl_generics #ty_ident #ty_generics #where_clause {
                        #(#fns)*
                    }
                }
            }
            Fields::Unnamed(ref fields_unnamed) => {
                let fns = fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                    let acc_fn = format_ident!("acc_{}", i);
                    let fty = &f.ty;
                    let index = syn::Index::from(i);
                    quote! {
                        /// Accessor to the tuple field at index #i.
                        pub const fn #acc_fn() -> pathmod_core::Accessor<#ty_ident #ty_generics, #fty> {
                            let off = core::mem::offset_of!(#ty_ident #ty_generics, #index) as isize;
                            // SAFETY: `off` is computed from the field offset within the same allocation.
                            unsafe { pathmod_core::Accessor::<#ty_ident #ty_generics, #fty>::from_offset(off) }
                        }
                    }
                });
                quote! {
                    impl #impl_generics #ty_ident #ty_generics #where_clause {
                        #(#fns)*
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
    };

    TokenStream::from(gen)
}
