// use proc_macro::TokenStream;
// use quote::{format_ident, quote};
// use syn::{parse_macro_input, DeriveInput, Data, Fields, Meta, Lit};
//
// #[proc_macro_derive(Table, attributes(table, column))]
// pub fn derive_table(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let ident = input.ident.clone();
//
//     // #[table(name = "...")]
//     let mut table_name = ident.to_string().to_lowercase();
//     for attr in input.attrs.iter().filter(|a| a.path().is_ident("table")) {
//         if let Ok(Meta::List(ml)) = attr.parse_meta() {
//             for nm in ml.nested.iter() {
//                 if let syn::NestedMeta::Meta(Meta::NameValue(nv)) = nm {
//                     if nv.path.is_ident("name") {
//                         if let Lit::Str(s) = &nv.lit { table_name = s.value(); }
//                     }
//                 }
//             }
//         }
//     }
//
//     // Collect fields
//     let mut field_idents = Vec::new();
//     let mut field_types  = Vec::new();
//     let mut col_name_idents = Vec::new();
//     let mut col_name_literals = Vec::new();
//
//     if let Data::Struct(d) = &input.data {
//         if let Fields::Named(named) = &d.fields {
//             for f in &named.named {
//                 let fname = f.ident.clone().expect("named field");
//                 let fty = f.ty.clone();
//                 let mut col_name = fname.to_string();
//                 for attr in f.attrs.iter().filter(|a| a.path().is_ident("column")) {
//                     if let Ok(Meta::List(ml)) = attr.parse_meta() {
//                         for nm in ml.nested.iter() {
//                             if let syn::NestedMeta::Meta(Meta::NameValue(nv)) = nm {
//                                 if nv.path.is_ident("name") {
//                                     if let Lit::Str(s) = &nv.lit { col_name = s.value(); }
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 field_idents.push(fname.clone());
//                 field_types.push(fty);
//                 let tag_ident = format_ident!("{}{}Tag", ident, pascal(&fname.to_string()));
//                 col_name_idents.push(tag_ident);
//                 col_name_literals.push(col_name);
//             }
//         } else { panic!("#[derive(Table)] requires named fields"); }
//     } else { panic!("#[derive(Table)] only for structs"); }
//
//     let proxy_ident = format_ident!("{}Proxy", ident);
//     let nproxy_ident = format_ident!("{}NullableProxy", ident);
//
//     let gen = quote! {
//         // Column tag structs for names
//         #(pub struct #col_name_idents; impl sqla::col::ColumnMeta for #col_name_idents { const NAME: &'static str = #col_name_literals; })*
//
//         // Proxies
//         pub struct #proxy_ident { #(pub #field_idents: sqla::col::Col<#ident, #field_types, sqla::types::NotNull, #col_name_idents>,)* }
//         pub struct #nproxy_ident { #(pub #field_idents: sqla::col::Col<#ident, #field_types, sqla::types::Nullable, #col_name_idents>,)* }
//
//         impl sqla::table::TableMeta for #ident {
//             type Proxy = #proxy_ident;
//             type NullableProxy = #nproxy_ident;
//             const NAME: &'static str = #table_name;
//             fn proxy() -> Self::Proxy { #proxy_ident { #( #field_idents: sqla::col::Col { _p: core::marker::PhantomData }, )* } }
//             fn nullable_proxy() -> Self::NullableProxy { #nproxy_ident { #( #field_idents: sqla::col::Col { _p: core::marker::PhantomData }, )* } }
//         }
//     };
//     gen.into()
// }
//
// fn pascal(s: &str) -> String {
//     let mut out = String::new();
//     let mut up = true;
//     for ch in s.chars() {
//         if ch == '_' { up = true; continue; }
//         if up { out.extend(ch.to_uppercase()); up = false; } else { out.push(ch); }
//     }
//     out
// }
//
