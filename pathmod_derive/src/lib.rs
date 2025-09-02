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
                            // Compute offset using a null pointer projection in a const-friendly way.
                            // Note: using pointer arithmetic without dereferencing.
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
            _ => {
                let msg = "#[derive(Accessor)] currently supports only named-field structs";
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
