// use std::env::current_dir;

// use proc_macro::TokenStream;
// use quote::quote;
// use self_rust_tokenize::SelfRustTokenize;
// use syn::{parse_macro_input, LitStr};
// use htn::prelude::*;

// #[proc_macro]
// pub fn embed_htn(input: TokenStream) -> TokenStream {
// 	let src = parse_macro_input!(input as LitStr).value();

// 	// let src = std::fs::read_to_string(current_dir().unwrap().join(src)).unwrap();

// 	let ir = htn::parsing::lexer::htn_lexer().parse(&src);
// 	for err in ir.errors() {
// 		println!("{}", err);
// 	}
// 	let Some(ir) = ir.output() else {
// 		panic!("Failed to lex HTN source due to errors:\n{}", ir.errors().map(|e| format!("{e}\n")).collect::<String>());
// 	};
// 	let obj = htn::parsing::htn_parser().parse(&**ir);
// 	for err in obj.errors() {
// 		println!("{}", err);
// 	}
// 	let Some(obj): Option<Vec<_>> = obj.output().cloned() else {
// 		panic!("Failed to parse HTN source due to errors:\n{}", obj.errors().map(|e| format!("{e}\n")).collect::<String>());
// 	};
// 	let bytecode = htn::parsing::emitter::emit(obj);
// 	let tokens = bytecode.into_iter().map(|op| op.to_tokens());

// 	// let bytecode_tokens

// 	let output = quote! {
// 		{
// 			use htn::prelude::*;
// 			use embed_requirements::*;
// 			&[#(#tokens),*]
// 		}
// 	};

// 	output.into()
// }

// #[proc_macro_attribute]
// pub fn enum_dispatched(_: TokenStream, input: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(input as ItemEnum);
    
//     let ident = &input.ident;

//     let mut output = proc_macro2::TokenStream::new();

//     for var in input.variants.iter_mut() {
//         output.append_all(quote!(pub struct #var));

//         var.attrs.clear();
//         var.fields = Fields::Unnamed(syn::FieldsUnnamed::parse(&format!("({})", var.ident)).unwrap());
//     }

//     output.into()
// }

// #[proc_macro_derive(IterEnum)]
// pub fn iter_enum(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as ItemEnum);
//     let ident = &input.ident;
//     if input.variants.iter().any(|v| !matches!(v.fields, Fields::Unit)) {
//         panic!("IterEnum only supports unit variants");
//     }
//     let variants = input.variants.iter().map(|v| &v.ident);
    
//     let output = quote! {
//         impl IterEnum for #ident {
//             fn variants() -> &'static [Self] {
//                 &[#(Self::#variants),*]
//             }
//         }
//     };

// 	output.into()
// }

// #[proc_macro_attribute]
// pub fn ratios(_: TokenStream, input: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(input as ItemEnum);
    
//     let ident = &input.ident;

//     let mut variants: Vec<Ident> = Vec::with_capacity(input.variants.len());
//     let mut ratios: Vec<LitInt> = Vec::with_capacity(input.variants.len());

//     for var in input.variants.iter_mut() {
//         let attr = &mut var.attrs.iter_mut().find(|attr| attr.path().is_ident("ratio")).expect("Missing ratio attribute");
//         let ratio = attr.parse_args::<LitInt>().expect("Invalid ratio attribute");

//         variants.push(var.ident.clone());
//         ratios.push(ratio);

//         var.attrs.retain(|attr| !attr.path().is_ident("ratio"));
//     }

//     quote! {
//         #input

//         impl #ident {
//             pub fn get_ratio(&self) -> usize {
//                 match self {
//                     #(Self::#variants => #ratios,)*
//                 }
//             }
//         }
//     }.into()
// }
