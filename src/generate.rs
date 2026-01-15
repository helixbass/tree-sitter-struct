use std::collections::HashMap;

use heck::ToPascalCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::grammar_json::{Choice, Root, Rule};

pub type SnakeCaseName = String;

pub fn generate(root: &Root, language: &str, string_literals: &HashMap<String, SnakeCaseName>) {
    let rules = root
        .rules
        .iter()
        .filter(|(rule_name, _)| !rule_name.starts_with("_"))
        .map(|(rule_name, rule)| match rule {
            Rule::Seq(seq) => {
                let struct_name = rule_name.to_pascal_case();
                let struct_fields = seq
                    .members
                    .iter()
                    .map(|member| get_struct_field(member))
                    .collect::<Vec<_>>();
                quote! {
                    pub struct #struct_name {
                        #(#struct_fields)*
                    }
                }
            }
            Rule::String(string) => {
                let struct_name = rule_name.to_pascal_case();
                let struct_field_name = &string_literals[&string.value];
                let struct_field_type = string_literals[&string.value].to_pascal_case();
                let struct_field_type = format_ident!("{}", struct_field_type);
                let struct_fields = vec![print_struct_field(
                    &format_ident!("{}", struct_field_name),
                    quote! { #struct_field_type },
                )];
                quote! {
                    pub struct #struct_name {
                        #(#struct_fields)*
                    }
                }
            }
            Rule::Choice(choice) => {
                let enum_name = rule_name.to_pascal_case();
                let enum_variants_and_structs = choice
                    .members
                    .iter()
                    .map(|member| get_enum_variant_and_struct(member))
                    .collect::<Vec<_>>();
                let enum_variants = enum_variants_and_structs.iter().map(|(variant, _)| variant);
                quote! {
                    pub enum #enum_name {
                        #(#enum_variants),*
                    }
                }
            }
            rule => unimplemented!("rule: {rule:#?}"),
        })
        .collect::<Vec<_>>();
}

fn get_struct_field(rule: &Rule) -> TokenStream {
    match rule {
        Rule::Choice(choice) if is_option(choice) => {
            let struct_field_type = get_type(&choice.members[0]);
            print_struct_field(
                &format_ident!("{}", get_struct_field_name(&choice.members[0])),
                quote! { Option<#struct_field_type> },
            )
        }
        Rule::Repeat(repeat) => {
            let item_type = get_type(&repeat.content);
            print_struct_field(
                &format_ident!("{}", get_struct_field_name(rule)),
                quote! { Vec<#item_type> },
            )
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_enum_variant_and_struct(rule: &Rule) -> (TokenStream, TokenStream) {
    match rule {
        Rule::Seq(seq) => {
            unimplemented!()
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_type(rule: &Rule) -> TokenStream {
    match rule {
        Rule::Symbol(symbol) => {
            let type_ = symbol.name.to_pascal_case();
            quote! { #type_ }
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_struct_field_name(rule: &Rule) -> String {
    match rule {
        Rule::Symbol(symbol) => symbol.name.clone(),
        Rule::Repeat(repeat) => pluralize(&get_struct_field_name(&repeat.content)),
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn print_struct_field(struct_field_name: &Ident, struct_field_type: TokenStream) -> TokenStream {
    quote! {
        pub #struct_field_name: #struct_field_type
    }
}

fn is_option(choice: &Choice) -> bool {
    choice.members.len() == 2 && choice.members[1].is_blank() && !choice.members[0].is_blank()
}

fn pluralize(name: &str) -> String {
    format!("{name}s")
}
