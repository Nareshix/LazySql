use proc_macro2::{Span, TokenStream};
use quote::quote;
use sqlitex_type_inference::{
    binding_patterns::BindingParam, expr::BaseType, table::ColumnInfo, QueryCardinality,
};

pub fn generate_read_methods(
    sql_span: Span,
    struct_name: &syn::Ident,
    ident: &syn::Ident,
    field_attrs: &[syn::Attribute],
    doc_comment: &str,
    binding_types: &[BindingParam],
    param_names: &[String],
    select_types: &[ColumnInfo],
    cardinality: QueryCardinality,
) -> syn::Result<(TokenStream, TokenStream)> {
    let mut generated_structs = quote! {};
    let mut generated_methods = quote! {};

    let is_single_col = select_types.len() == 1;

    let method_name = ident.to_string();
    let pascal_name: String = method_name
        .split('_')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();

    // PREFIXED to prevent naming collisions across identical methods in different tables
    let output_struct_name = quote::format_ident!("{}{}", struct_name, pascal_name);
    let mapper_struct_name = quote::format_ident!("{}{}_", struct_name, pascal_name);
    let scalar_mapper_name = quote::format_ident!("{}_{}_scalar_", struct_name, ident);

    let mut method_args = Vec::new();
    let mut bind_calls = Vec::new();

    for (i, bind_param) in binding_types.iter().enumerate() {
        let arg_name = quote::format_ident!("{}", param_names[i]);
        let bind_type = &bind_param.data_type;
        let bind_index = (i + 1) as i32;

        let rust_base_type = match bind_type.base_type {
            BaseType::Integer => quote! { i64 },
            BaseType::Real => quote! { f64 },
            BaseType::Bool => quote! { bool },
            BaseType::Text => quote! { &str },
            BaseType::Blob => quote! { &[u8] },
            _ => return Err(syn::Error::new(sql_span, "Unable to infer type for `?`. Consider casting with `::` or `CAST AS`")),
        };

        let final_type = if bind_type.nullable {
            quote! { Option<#rust_base_type> }
        } else {
            quote! { #rust_base_type }
        };

        method_args.push(quote! { #arg_name: #final_type });
        bind_calls.push(quote! {
            preparred_statement.bind_parameter(#bind_index, #arg_name)?;
        });
    }

    let single_col_rust_type = if is_single_col {
        let col = &select_types[0];
        let base_ty = match col.data_type.base_type {
            BaseType::Integer => quote! { i64 },
            BaseType::Real => quote! { f64 },
            BaseType::Text => quote! { String },
            BaseType::Blob => quote! { Vec<u8> },
            BaseType::Bool => quote! { bool },
            _ => quote! { i64 }, // fallback
        };
        if col.data_type.nullable {
            quote! { Option<#base_ty> }
        } else {
            quote! { #base_ty }
        }
    } else {
        quote! {}
    };

    if !is_single_col || cardinality == QueryCardinality::MaybeMany {
        let mut struct_fields = Vec::new();
        for col in select_types.iter() {
            let name = quote::format_ident!("{}", col.name);
            let base_ty = match col.data_type.base_type {
                BaseType::Integer => quote! { i64 },
                BaseType::Real => quote! { f64 },
                BaseType::Text => quote! { String },
                BaseType::Blob => quote! { Vec<u8> },
                BaseType::Bool => quote! { bool },
                _ => return Err(syn::Error::new(sql_span, "Unable to infer return type. Consider casting with `::` or `CAST AS`")),
            };
            let final_ty = if col.data_type.nullable {
                quote! { Option<#base_ty> }
            } else {
                quote! { #base_ty }
            };
            struct_fields.push(quote! { pub #name: #final_ty });
        }
        generated_structs.extend(quote! {
            #[derive(Clone, Debug, sqlitex::SqlMapping)]
            pub struct #output_struct_name {
                #(#struct_fields),*
            }
        });
    }

    if is_single_col && cardinality != QueryCardinality::MaybeMany {
        let col = &select_types[0];
        let is_nullable = col.data_type.nullable;
        let base_ty = match col.data_type.base_type {
            BaseType::Integer => quote! { i64 },
            BaseType::Real => quote! { f64 },
            BaseType::Text => quote! { String },
            BaseType::Blob => quote! { Vec<u8> },
            BaseType::Bool => quote! { bool },
            _ => quote! { i64 },
        };

        if is_nullable {
            generated_structs.extend(quote! {
                #[allow(non_camel_case_types)]
                #[derive(Clone)]
                struct #scalar_mapper_name;
                impl sqlitex::traits::row_mapper::RowMapper for #scalar_mapper_name {
                    type Output = Option<#base_ty>;
                    unsafe fn map_row(&self, stmt: *mut sqlitex::libsqlite3_sys::sqlite3_stmt) -> Option<#base_ty> {
                        <Option<#base_ty> as sqlitex::traits::from_sql::FromSql>::from_sql(stmt, 0)
                    }
                }
            });
        } else {
            generated_structs.extend(quote! {
                #[allow(non_camel_case_types)]
                #[derive(Clone)]
                struct #scalar_mapper_name;
                impl sqlitex::traits::row_mapper::RowMapper for #scalar_mapper_name {
                    type Output = #base_ty;
                    unsafe fn map_row(&self, stmt: *mut sqlitex::libsqlite3_sys::sqlite3_stmt) -> #base_ty {
                        <#base_ty as sqlitex::traits::from_sql::FromSql>::from_sql(stmt, 0)
                    }
                }
            });
        }
    }

    let prepare_block = quote! {
        if self.#ident.stmt.is_null() {
            unsafe {
                sqlitex::utility::utils::prepare_stmt(
                    self.__db.db,
                    &mut self.#ident.stmt,
                    self.#ident.sql_query
                )?;
            }
        }
        let mut preparred_statement = sqlitex::internal_sqlite::preparred_statement::PreparredStmt {
            stmt: self.#ident.stmt,
            conn: self.__db.db,
        };
        #(#bind_calls)*
    };

    let ret_err_type = if binding_types.is_empty() {
        quote! { sqlitex::errors::SqlReadError }
    } else {
        quote! { sqlitex::errors::SqlReadErrorBindings }
    };

    match (cardinality, is_single_col) {
        (QueryCardinality::MaybeMany, _) => {
            generated_methods.extend(quote! {
                #(#field_attrs)*
                #[doc = #doc_comment]
                pub fn #ident(&mut self #(, #method_args)*) -> Result<sqlitex::internal_sqlite::rows_dao::Rows<'_, #mapper_struct_name>, #ret_err_type> {
                    #prepare_block
                    Ok(preparred_statement.query(#output_struct_name))
                }
            });
        }
        (QueryCardinality::ZeroOrOne, false) => {
            generated_methods.extend(quote! {
                #(#field_attrs)*
                #[doc = #doc_comment]
                pub fn #ident(&mut self #(, #method_args)*) -> Result<Option<#output_struct_name>, sqlitex::errors::Error> {
                    #prepare_block
                    preparred_statement.query(#output_struct_name)
                        .first()
                        .map_err(sqlitex::errors::Error::from)
                }
            });
        }
        (QueryCardinality::ExactlyOne, false) => {
            generated_methods.extend(quote! {
                #(#field_attrs)*
                #[doc = #doc_comment]
                pub fn #ident(&mut self #(, #method_args)*) -> Result<#output_struct_name, sqlitex::errors::Error> {
                    #prepare_block
                    preparred_statement.query(#output_struct_name)
                        .first()
                        .map_err(sqlitex::errors::Error::from)
                        .map(|opt| opt.expect("aggregate query must return exactly one row"))
                }
            });
        }
        (QueryCardinality::ZeroOrOne, true) => {
            generated_methods.extend(quote! {
                #(#field_attrs)*
                #[doc = #doc_comment]
                pub fn #ident(&mut self #(, #method_args)*) -> Result<Option<#single_col_rust_type>, sqlitex::errors::Error> {
                    #prepare_block
                    preparred_statement.query(#scalar_mapper_name)
                        .first()
                        .map_err(sqlitex::errors::Error::from)
                }
            });
        }
        (QueryCardinality::ExactlyOne, true) => {
            generated_methods.extend(quote! {
                #(#field_attrs)*
                #[doc = #doc_comment]
                pub fn #ident(&mut self #(, #method_args)*) -> Result<#single_col_rust_type, sqlitex::errors::Error> {
                    #prepare_block
                    preparred_statement.query(#scalar_mapper_name)
                        .first()
                        .map_err(sqlitex::errors::Error::from)
                        .map(|opt| opt.expect("aggregate query must return exactly one row"))
                }
            });
        }
    }

    Ok((generated_structs, generated_methods))
}