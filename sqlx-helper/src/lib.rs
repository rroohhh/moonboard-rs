// lolo
extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream as ExternTokenStream;
use quote::quote;
use syn;
use syn::visit_mut::VisitMut;

#[derive(Debug, FromMeta)]
struct ArgsFromAttrs {
    #[darling(default)]
    table_name: Option<String>,
}

#[proc_macro_attribute]
pub fn insert(_attr: ExternTokenStream, item: ExternTokenStream) -> ExternTokenStream {
    item
}

#[proc_macro_attribute]
pub fn insertable(attr: ExternTokenStream, item: ExternTokenStream) -> ExternTokenStream {
    let attr_args = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let mut item = syn::parse_macro_input!(item as syn::ItemStruct);

    let args = match ArgsFromAttrs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let struct_name = item.ident.clone();

    let table_name = proc_macro2::Ident::new(
        &args
            .table_name
            .unwrap_or(struct_name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );

    let mut remover = FieldAttrRemover::new();
    remover.visit_item_struct_mut(&mut item);

    let my_fields = remover.fields;

    let mut field_names = vec![];
    let mut field_access = vec![];
    let mut embedded_field_access = vec![];
    let mut embedded_field_insert_stmt = vec![];

    for (name, args) in my_fields {
        if args.skip {
            continue;
        }

        match args.embed_with {
            Some(module) => {
                let module = syn::Ident::new(&module, proc_macro2::Span::call_site());

                embedded_field_access.push(quote! {
                    &$value.#name
                });

                match args.embed_translator {
                    Some(translator) => {
                        let translator =
                            syn::Ident::new(&translator, proc_macro2::Span::call_site());

                        embedded_field_insert_stmt.push(quote! {
                            $crate::#module::insert!($crate::#translator(&$value, v), |$query| $execute);
                        })
                    }
                    None => embedded_field_insert_stmt.push(quote! {
                        $crate::#module::insert!(v, |$query| $execute);
                    }),
                }
            }
            None => {
                assert!(
                    args.embed_translator.is_none(),
                    "cannot use embed_translator without embed_with"
                );
                field_names.push(name.clone());

                field_access.push(match args.with {
                    Some(accessor) => {
                        let accessor = syn::Ident::new(&accessor, proc_macro2::Span::call_site());

                        quote! {
                            $crate::#accessor($value.#name)
                        }
                    }
                    None => quote! {
                        $value.#name
                    },
                });
            }
        }
    }

    let mut insert_statement = "INSERT INTO ".to_string();
    insert_statement.push_str(&table_name.to_string());
    insert_statement.push_str(" (");

    if field_names.len() > 0 {
        insert_statement.push_str(&field_names[0].to_string());

        for field in &field_names[1..] {
            insert_statement.push_str(", ");
            insert_statement.push_str(&field.to_string());
        }
    }

    insert_statement.push_str(") VALUES (");

    if field_names.len() > 0 {
        insert_statement.push_str("$1");

        for i in 2..(field_names.len() + 1) {
            insert_statement.push_str(", ");
            insert_statement.push_str(&format!("${}", i));
        }
    }

    insert_statement.push_str(")");

    let insert_name = syn::Ident::new(
        &("_".to_owned() + &table_name.to_string() + "_insert"),
        proc_macro2::Span::call_site(),
    );

    let more = quote!(
        #item

        pub mod #table_name {
            use super::*;

            #[macro_export]
            macro_rules! #insert_name {
                ($value:expr, |$query:ident| $execute:block) => {
                    #(for v in #embedded_field_access {
                        #embedded_field_insert_stmt
                    })*

                    let $query = sqlx::query!(#insert_statement, #(#field_access),*);
                    $execute;
                }
            }

            // cool hack, see https://github.com/SergioBenitez/Rocket/issues/19#issuecomment-453822603
            pub use #insert_name as insert;
        }
    );

    more.into()
}

#[derive(Debug, FromMeta, Default)]
struct FieldArgs {
    #[darling(default)]
    with: Option<String>,
    #[darling(default)]
    embed_with: Option<String>,
    #[darling(default)]
    embed_translator: Option<String>,
    #[darling(default)]
    skip: bool,
}

#[derive(Debug)]
struct FieldAttrRemover {
    fields: Vec<(syn::Ident, FieldArgs)>,
}

impl FieldAttrRemover {
    fn new() -> Self {
        Self { fields: vec![] }
    }
}

const mod_name: &str = "sqlx_helper";
const blacklist: &[&str] = &["insert"];

impl syn::visit_mut::VisitMut for FieldAttrRemover {
    fn visit_field_mut(&mut self, field: &mut syn::Field) {
        let name = field.ident.as_ref().unwrap();

        let (other_attrs, my_attrs): (_, Vec<_>) =
            field
                .attrs
                .iter()
                .cloned()
                .partition(|a| match a.path.get_ident() {
                    Some(i) => !blacklist.contains(&i.to_string().as_str()),
                    None => {
                        let mut found_leading = false;

                        for s in a.path.segments.iter() {
                            if &s.ident.to_string() == mod_name {
                                found_leading = true;
                            } else if found_leading
                                && blacklist.contains(&s.ident.to_string().as_str())
                            {
                                return false;
                            } else {
                                found_leading = false;
                            }
                        }

                        true
                    }
                });

        if my_attrs.len() > 0 {
            for attr in my_attrs {
                let field_args = attr.parse_meta().unwrap();

                let args = FieldArgs::from_meta(&field_args).unwrap();

                self.fields.push((name.clone(), args));

                break;
            }
        } else {
            self.fields.push((name.clone(), Default::default()));
        }

        field.attrs = other_attrs;
    }
}
