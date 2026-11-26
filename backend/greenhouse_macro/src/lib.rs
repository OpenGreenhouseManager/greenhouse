extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(IntoJsonResponse)]
pub fn into_response_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate.
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation.
    impl_into_response_macro(&ast)
}

fn impl_into_response_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generated = quote! {

        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                axum::Json(self).into_response()
            }
        }
    };
    generated.into()
}
