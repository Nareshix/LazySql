use proc_macro2::TokenStream;
use quote::quote;
use crate::codegen::context::CodegenContext;

pub fn generate_create_table(ctx: &CodegenContext) -> TokenStream {
    let ident = ctx.ident;
    let field_attrs = ctx.field_attrs;
    let doc_comment = &ctx.doc_comment;

    let prepare_block = ctx.generate_prepare_block();

    quote! {
        #(#field_attrs)*
        #[doc = #doc_comment]
        pub fn #ident(&mut self) -> Result<(), sqlitex::errors::SqlWriteError> {
            #prepare_block
            preparred_statement.step()?;
            Ok(())
        }
    }
}