extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Accessor)]
pub fn accessor_derive(_input: TokenStream) -> TokenStream {
    // Minimal no-op derive to allow compilation; functionality will be implemented later.
    let _ = parse_macro_input!(_input as DeriveInput);
    TokenStream::from(quote! {})
}
