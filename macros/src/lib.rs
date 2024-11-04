use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, Ident, ItemEnum, LitInt};

#[proc_macro_derive(IterEnum)]
pub fn iter_enum(input: TokenStream) -> TokenStream {
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

#[proc_macro_attribute]
pub fn ratios(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemEnum);
    
    let ident = &input.ident;

    let mut variants: Vec<Ident> = Vec::with_capacity(input.variants.len());
    let mut ratios: Vec<LitInt> = Vec::with_capacity(input.variants.len());

    for var in input.variants.iter_mut() {
        let attr = &mut var.attrs.iter_mut().find(|attr| attr.path().is_ident("ratio")).expect("Missing ratio attribute");
        let ratio = attr.parse_args::<LitInt>().expect("Invalid ratio attribute");

        variants.push(var.ident.clone());
        ratios.push(ratio);

        var.attrs.retain(|attr| !attr.path().is_ident("ratio"));
    }

    quote! {
        #input

        impl #ident {
            pub fn get_ratio(&self) -> usize {
                match self {
                    #(Self::#variants => #ratios,)*
                }
            }
        }
    }.into()
}
