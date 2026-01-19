use std::collections::HashMap;
use std::fs;

use heck::ToPascalCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::grammar_json::{Choice, Root, Rule};

pub type SnakeCaseName = String;

pub fn generate(root: &Root, language: &str, string_literals: &HashMap<String, SnakeCaseName>) {
    let rules = root
        .rules
        .iter()
        .map(|(rule_name, rule)| {
            get_struct_or_enum(rule, Some(rule_name.clone()), None, string_literals).0
        })
        .collect::<Vec<_>>();
    let code = quote! {
        #(#rules)*
    }
    .to_string();
    fs::write("tmp-out.rs", code).unwrap();
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
                .map(|member| {
                    get_struct_field_and_struct_or_enum(member, &struct_name, string_literals)
                })
                .collect::<Vec<_>>();
            (
                {
                    let printed_struct = print_struct(
                        &format_ident!("{struct_name}"),
                        &struct_fields
                            .iter()
                            .map(|(struct_field, _)| struct_field.clone())
                            .collect::<Vec<_>>(),
                    );
                    let sub_struct_and_enum_types = struct_fields
                        .iter()
                        .map(|(_, struct_or_enum_type)| struct_or_enum_type.clone())
                        .collect::<Vec<_>>();
                    quote! {
                        #printed_struct
                        #(#sub_struct_and_enum_types)*
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
                print_struct(&format_ident!("{struct_name}"), &struct_fields),
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
            let enum_variants = enum_variants_and_structs
                .iter()
                .map(|(variant, _)| variant.clone())
                .collect::<Vec<_>>();
            (
                {
                    let enum_ = print_enum(&format_ident!("{enum_name}"), &enum_variants);
                    let enum_variant_struct_or_enums = enum_variants_and_structs
                        .iter()
                        .map(|(_, struct_or_enum)| struct_or_enum);
                    quote! {
                        #enum_

                        #(#enum_variant_struct_or_enums)*
                    }
                },
                enum_name,
            )
        }
        Rule::Field(field) => {
            match &*field.content {
                Rule::Choice(_) => {
                    get_struct_or_enum(
                        &field.content,
                        // TODO: this is ugly, this is semantically overriding
                        // `rule_name`, really I guess I'm just trying to
                        // control the generated struct name for this `field()`
                        // value?
                        Some(field.name.clone()),
                        Some(struct_or_enum_prefix),
                        string_literals,
                    )
                }
                rule => unimplemented!("rule: {rule:#?}"),
            }
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn print_enum(enum_name: &Ident, enum_variants: &[TokenStream]) -> TokenStream {
    quote! {
        pub enum #enum_name {
            #(#enum_variants),*
        }
    }
}

fn print_struct(struct_name: &Ident, struct_fields: &[TokenStream]) -> TokenStream {
    quote! {
        pub struct #struct_name {
            #(#struct_fields),*
        }
    }
}

fn get_struct_field_and_struct_or_enum(
    rule: &Rule,
    parent_struct_name: &str,
    string_literals: &HashMap<String, SnakeCaseName>,
) -> (TokenStream, TokenStream) {
    match rule {
        Rule::Choice(choice) if is_option(choice) => {
            let struct_field_type = get_type(&choice.members[0]);
            (
                print_struct_field(
                    &format_ident!("{}", get_struct_field_name(&choice.members[0])),
                    quote! { Option<#struct_field_type> },
                ),
                quote! {},
            )
        }
        Rule::Repeat(repeat) => {
            let item_type = get_type(&repeat.content);
            (
                print_struct_field(
                    &format_ident!("{}", get_struct_field_name(rule)),
                    quote! { Vec<#item_type> },
                ),
                quote! {},
            )
        }
        Rule::Symbol(symbol) => {
            let item_type = get_type(rule);
            (
                print_struct_field(
                    &format_ident!("{}", without_leading_underscore(&symbol.name)),
                    quote! { #item_type },
                ),
                quote! {},
            )
        }
        Rule::String(string) => {
            let struct_field_name = format_ident!("{}", string_literals[&string.value]);
            let struct_field_type = string_literals[&string.value].to_pascal_case();
            let struct_field_type = format_ident!("{}", struct_field_type);
            (
                print_struct_field(&struct_field_name, quote! { #struct_field_type }),
                quote! {},
            )
        }
        Rule::Field(field) => {
            let struct_field_name = format_ident!("{}", field.name);
            let (struct_field_type, struct_field_struct_or_enum) =
                get_struct_field_field_type_and_struct_or_enum(
                    parent_struct_name,
                    rule,
                    string_literals,
                );
            (
                print_struct_field(&struct_field_name, quote! { #struct_field_type }),
                struct_field_struct_or_enum,
            )
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn get_struct_field_field_type_and_struct_or_enum(
    parent_struct_name: &str,
    rule: &Rule,
    string_literals: &HashMap<String, SnakeCaseName>,
) -> (TokenStream, TokenStream) {
    match &*rule.as_field().content {
        Rule::Choice(_) => {
            let (enum_, enum_name) =
                get_struct_or_enum(rule, None, Some(parent_struct_name), string_literals);
            (
                {
                    let enum_name = format_ident!("{enum_name}");
                    quote! { #enum_name }
                },
                enum_,
            )
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
            enum_variant_and_struct_or_enum(
                &enum_variant_struct_or_enum_name,
                parent_enum_name,
                enum_variant_struct_or_enum,
            )
        }
        Rule::Prec(prec) => {
            get_enum_variant_and_struct_or_enum(&prec.content, parent_enum_name, string_literals)
        }
        Rule::Symbol(symbol) => {
            let enum_variant_name = format_ident!(
                "{}",
                without_leading_underscore(&symbol.name).to_pascal_case()
            );
            (quote! { #enum_variant_name(#enum_variant_name) }, quote! {})
        }
        rule => unimplemented!("rule: {rule:#?}"),
    }
}

fn enum_variant_and_struct_or_enum(
    enum_variant_struct_or_enum_name: &str,
    parent_enum_name: &str,
    enum_variant_struct_or_enum: TokenStream,
) -> (TokenStream, TokenStream) {
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

fn get_type(rule: &Rule) -> TokenStream {
    match rule {
        Rule::Symbol(symbol) => {
            let type_ = format_ident!(
                "{}",
                without_leading_underscore(&symbol.name).to_pascal_case()
            );
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
        Rule::Symbol(symbol) => without_leading_underscore(&symbol.name),
        Rule::Repeat(repeat) => pluralize(&get_struct_field_name(&repeat.content)),
        Rule::Seq(seq) => get_struct_field_name(
            seq.members
                .iter()
                .find(|member| matches!(member, Rule::Symbol(_)))
                .expect("Couldn't find symbol in seq"),
        ),
        Rule::Field(field) => field.name.clone(),
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
