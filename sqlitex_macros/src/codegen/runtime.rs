use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::RuntimeSqlInput;

pub fn generate_runtime_method(
    ident: &syn::Ident,
    field_attrs: &[syn::Attribute],
    doc_comment: &str,
    runtime_input: &RuntimeSqlInput,
) -> TokenStream {
    let mut generated_methods = quote! {};
    let mut method_args = Vec::new();
    let mut bind_calls = Vec::new();

    for (i, arg_type) in runtime_input.args.iter().enumerate() {
        let arg_name = quote::format_ident!("arg_{}", i);
        let bind_index = (i + 1) as i32;

        method_args.push(quote! { #arg_name: #arg_type });

        bind_calls.push(quote! {
            preparred_statement.bind_parameter(#bind_index, #arg_name)?;
        });
    }

    if let Some(ret_type) = &runtime_input.return_type {
        let mapper_type = if let syn::Type::Path(type_path) = &ret_type {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                let primitives = [
                    "i64", "i32", "u64", "u32", "f64", "f32", "bool", "String", "Option",
                ];

                if primitives.iter().any(|&p| type_name.starts_with(p)) {
                    quote! { #ret_type }
                } else {
                    let new_ident = quote::format_ident!("{}_", segment.ident);
                    quote! { #new_ident }
                }
            } else {
                quote! { #ret_type }
            }
        } else {
            quote! { #ret_type }
        };

        generated_methods.extend(quote! {
            #(#field_attrs)*
            #[doc = #doc_comment]
            pub fn #ident(&mut self #(, #method_args)*) -> Result<sqlitex::internal_sqlite::rows_dao::Rows<'_, #mapper_type>, sqlitex::errors::SqlReadErrorBindings> {
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

                Ok(preparred_statement.query(#mapper_type))
            }
        });
    } else {
        generated_methods.extend(quote! {
            #(#field_attrs)*
            #[doc = #doc_comment]
            pub fn #ident(&mut self #(, #method_args)*) -> Result<(), sqlitex::errors::SqlWriteBindingError> {
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

                preparred_statement.step()?;
                Ok(())
            }
        });
    }

    generated_methods
}