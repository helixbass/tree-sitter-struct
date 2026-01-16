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
        .map(|(rule_name, rule)| {
            get_struct_or_enum(rule, Some(rule_name.clone()), None, string_literals).0
        })
        .collect::<Vec<_>>();
}

fn get_struct_or_enum(
    rule: &Rule,
    rule_name: Option<String>,
    struct_or_enum_prefix: Option<&str>,
    string_literals: &HashMap<String, SnakeCaseName>,
) -> (TokenStream, String) {
    let rule_name = rule_name.unwrap_or_else(|| get_struct_field_name(rule).to_pascal_case());
    let struct_or_enum_prefix = struct_or_enum_prefix.unwrap_or("");
    match rule {
        Rule::Seq(seq) => {
            let struct_name = format!("{struct_or_enum_prefix}{}", rule_name.to_pascal_case());
            let struct_fields = seq
                .members
                .iter()
                .map(|member| get_struct_field(member, string_literals))
                .collect::<Vec<_>>();
            (
                quote! {
                    pub struct #struct_name {
                        #(#struct_fields)*
                    }
                },
                struct_name,
            )
        }
        Rule::String(string) => {
            let struct_name = format!("{struct_or_enum_prefix}{}", rule_name.to_pascal_case());
            let struct_field_name = &string_literals[&string.value];
            let struct_field_type = string_literals[&string.value].to_pascal_case();
            let struct_field_type = format_ident!("{}", struct_field_type);
            let struct_fields = vec![print_struct_field(
                &format_ident!("{}", struct_field_name),
                quote! { #struct_field_type },
            )];
            (
                quote! {
                    pub struct #struct_name {
                        #(#struct_fields)*
                    }
                },
                struct_name,
            )
        }
        Rule::Choice(choice) => {
            let enum_name = format!("{struct_or_enum_prefix}{}", rule_name.to_pascal_case());
            let enum_variants_and_structs = choice
                .members
                .iter()
                .map(|member| {
                    get_enum_variant_and_struct_or_enum(member, &enum_name, string_literals)
                })
                .collect::<Vec<_>>();
            let enum_variants = enum_variants_and_structs.iter().map(|(variant, _)| variant);
            (
                quote! {
                    pub enum #enum_name {
                        #(#enum_variants),*
                    }
                },
                enum_name,
            )
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_struct_field(rule: &Rule, string_literals: &HashMap<String, SnakeCaseName>) -> TokenStream {
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
        Rule::Symbol(symbol) => {
            let item_type = get_type(rule);
            print_struct_field(
                &format_ident!("{}", without_leading_underscore(&symbol.name)),
                quote! { #item_type },
            )
        }
        Rule::String(string) => {
            let struct_field_name = format_ident!("{}", string_literals[&string.value]);
            let struct_field_type = string_literals[&string.value].to_pascal_case();
            let struct_field_type = format_ident!("{}", struct_field_type);
            print_struct_field(&struct_field_name, quote! { #struct_field_type })
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_enum_variant_and_struct_or_enum(
    rule: &Rule,
    parent_enum_name: &str,
    string_literals: &HashMap<String, SnakeCaseName>,
) -> (TokenStream, TokenStream) {
    match rule {
        Rule::Seq(seq) => {
            let (enum_variant_struct_or_enum, enum_variant_struct_or_enum_name) =
                get_struct_or_enum(rule, None, Some(parent_enum_name), string_literals);
            (
                {
                    assert!(enum_variant_struct_or_enum_name.starts_with(parent_enum_name));
                    let enum_variant_name =
                        enum_variant_struct_or_enum_name[parent_enum_name.len()..].to_owned();
                    let enum_variant_struct_or_enum_name =
                        format_ident!("{enum_variant_struct_or_enum_name}");
                    let enum_variant_name = format_ident!("{enum_variant_name}");
                    quote! {
                        #enum_variant_name(#enum_variant_struct_or_enum_name)
                    }
                },
                enum_variant_struct_or_enum,
            )
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_type(rule: &Rule) -> TokenStream {
    match rule {
        Rule::Symbol(symbol) => {
            let type_ = without_leading_underscore(&symbol.name).to_pascal_case();
            quote! { #type_ }
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn without_leading_underscore(name: &str) -> String {
    if name.starts_with("_") {
        name[1..].to_owned()
    } else {
        name.to_owned()
    }
}

fn get_struct_field_name(rule: &Rule) -> String {
    match rule {
        Rule::Symbol(symbol) => symbol.name.clone(),
        Rule::Repeat(repeat) => pluralize(&get_struct_field_name(&repeat.content)),
        Rule::Seq(seq) => get_struct_field_name(
            seq.members
                .iter()
                .find(|member| matches!(member, Rule::Symbol(_)))
                .expect("Couldn't find symbol in seq"),
        ),
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
