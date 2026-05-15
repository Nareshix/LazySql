use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_create_table(
    ident: &syn::Ident,
    field_attrs: &[syn::Attribute],
    doc_comment: &str,
) -> TokenStream {
    quote! {
        #(#field_attrs)*
        #[doc = #doc_comment]
        pub fn #ident(&mut self) -> Result<(), sqlitex::errors::SqlWriteError> {
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
            preparred_statement.step()?;
            Ok(())
        }
    }
}