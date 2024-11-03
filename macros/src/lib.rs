use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemEnum};

#[proc_macro_derive(IterEnum)]
pub fn into(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let ident = &input.ident;
    if input.variants.iter().any(|v| !matches!(v.fields, Fields::Unit)) {
        panic!("IterEnum only supports unit variants");
    }
    let variants = input.variants.iter().map(|v| &v.ident);
    
    let output = quote! {
        impl IterEnum for #ident {
            fn variants() -> &'static [Self] {
                &[#(Self::#variants),*]
            }
        }
    };

	output.into()
}
