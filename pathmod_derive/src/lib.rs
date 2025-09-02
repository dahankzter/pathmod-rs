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

fn expand_enum(input: DeriveInput) -> proc_macro2::TokenStream {
    // Note: Keep control flow linear to help coverage tools attribute regions cleanly.
    let ty_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match input.data {
        Data::Enum(en) => {
            // Build method sets per variant for single-field variants only
            let mut per_variant_tokens = Vec::new();
            let mut error_msg: Option<&'static str> = None;
            for v in en.variants.iter() {
                let v_ident = &v.ident;
                match &v.fields {
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        let fty = &fields.unnamed.first().unwrap().ty;
                        let is_fn = format_ident!("is_{}", v_ident.to_string().to_lowercase());
                        let as_fn = format_ident!("as_{}", v_ident.to_string().to_lowercase());
                        let as_mut_fn =
                            format_ident!("as_{}_mut", v_ident.to_string().to_lowercase());
                        let set_fn = format_ident!("set_{}", v_ident.to_string().to_lowercase());
                        let map_fn = format_ident!("map_{}", v_ident.to_string().to_lowercase());
                        per_variant_tokens.push(quote! {
                            #[inline]
                            pub fn #is_fn(&self) -> bool { matches!(self, Self::#v_ident(_)) }
                            #[inline]
                            pub fn #as_fn(&self) -> Option<& #fty> { if let Self::#v_ident(ref v) = self { Some(v) } else { None } }
                            #[inline]
                            pub fn #as_mut_fn(&mut self) -> Option<&mut #fty> { if let Self::#v_ident(ref mut v) = self { Some(v) } else { None } }
                            #[inline]
                            pub fn #set_fn(&mut self, val: #fty) { *self = Self::#v_ident(val); }
                            #[inline]
                            pub fn #map_fn(&mut self, f: impl FnOnce(&mut #fty)) { if let Self::#v_ident(ref mut v) = self { f(v); } }
                        });
                    }
                    Fields::Named(fields) if fields.named.len() == 1 => {
                        let _ = &fields; // keep pattern usage without warnings
                        error_msg = Some("#[derive(EnumAccess)] currently supports only tuple variants with exactly one field; named-field single variants are not yet supported");
                        break;
                    }
                    Fields::Unit => {
                        error_msg = Some(
                            "#[derive(EnumAccess)] does not support unit variants in this MVP",
                        );
                        break;
                    }
                    _ => {
                        error_msg = Some("#[derive(EnumAccess)] supports only tuple variants with exactly one field");
                        break;
                    }
                }
            }
            if let Some(msg) = error_msg {
                return quote! { compile_error!(#msg); };
            }
            quote! {
                impl #impl_generics #ty_ident #ty_generics #where_clause {
                    #(#per_variant_tokens)*
                }
            }
        }
        _ => {
            quote! { compile_error!("#[derive(EnumAccess)] can only be used on enums"); }
        }
    }
}

#[proc_macro_derive(EnumAccess)]
pub fn enum_access_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let ts = expand_enum(input);
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

    // Additional unit tests for EnumAccess derive expansion
    #[test]
    fn enum_access_positive_single_field_tuple_variant() {
        let di: DeriveInput = parse_quote! { enum Msg { Int(i32), Text(String) } };
        let out = expand_enum(di);
        let s = out.to_string();
        // Check that methods for both variants appear
        assert!(s.contains("is_int"));
        assert!(s.contains("as_int"));
        assert!(s.contains("as_int_mut"));
        assert!(s.contains("set_int"));
        assert!(s.contains("map_int"));
        assert!(s.contains("is_text"));
        assert!(s.contains("as_text"));
        assert!(s.contains("as_text_mut"));
        assert!(s.contains("set_text"));
        assert!(s.contains("map_text"));
    }

    #[test]
    fn enum_access_error_on_unit_variant() {
        let di: DeriveInput = parse_quote! { enum E { A } };
        let out = expand_enum(di);
        let s = out.to_string();
        assert!(s.contains("compile_error") && s.contains("does not support unit variants"));
    }

    #[test]
    fn enum_access_error_on_multi_field_variant() {
        let di: DeriveInput = parse_quote! { enum E { Both(i32, i32) } };
        let out = expand_enum(di);
        let s = out.to_string();
        assert!(
            s.contains("compile_error")
                && s.contains("supports only tuple variants with exactly one field")
        );
    }

    #[test]
    fn enum_access_error_on_named_single_field_variant() {
        let di: DeriveInput = parse_quote! { enum E { V { v: i32 } } };
        let out = expand_enum(di);
        let s = out.to_string();
        assert!(
            s.contains("compile_error") && s.contains("currently supports only tuple variants")
        );
    }

    #[test]
    fn enum_access_error_on_non_enum() {
        let di: DeriveInput = parse_quote! { struct NotEnum { a: i32 } };
        let out = expand_enum(di);
        let s = out.to_string();
        assert!(s.contains("compile_error") && s.contains("can only be used on enums"));
    }

    // Exercise generics and where-clause propagation in struct expansion
    #[test]
    fn expands_generics_with_where_clause() {
        let di: DeriveInput = parse_quote! {
            struct Wrap<T: Clone, U>
            where
                U: core::fmt::Debug,
            {
                t: T,
                u: U,
            }
        };
        let out = expand(di);
        let s = out.to_string();
        // Accessors should be generated for both fields
        assert!(s.contains("acc_t"));
        assert!(s.contains("acc_u"));
        // The output token stream should mention where-clause Debug (stringly check)
        assert!(s.contains("Debug") || s.contains("where"));
    }

    // Exercise the zero-variant enum path (should generate an empty impl block)
    #[test]
    fn enum_access_empty_enum_generates_impl() {
        let di: DeriveInput = parse_quote! { enum Z {} };
        let out = expand_enum(di);
        let s = out.to_string();
        // Should produce an impl block for Z even if it contains no methods
        assert!(s.contains("impl") && s.contains("Z"));
    }
}
