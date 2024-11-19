use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

#[proc_macro_attribute]
pub fn attr_macro_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_none() {
        return quote_spanned! { input.sig.fn_token.span()=>
            compile_error!("fn must be `async fn`");
        }
        .into();
    }

    if input.sig.ident != "main" {
        return quote_spanned! { input.sig.ident.span()=>
            compile_error!("only `async fn main` can be used for #[wstd::main]");
        }
        .into();
    }

    if !input.sig.inputs.is_empty() {
        return quote_spanned! { input.sig.inputs.span()=>
            compile_error!("arguments to main are not supported");
        }
        .into();
    }
    let attrs = input.attrs;
    let output = input.sig.output;
    let block = input.block;
    quote! {
        pub fn main() #output {

            #(#attrs)*
            async fn __run() #output {
                #block
            }

            ::wstd::runtime::block_on(async {
                __run().await
            })
        }
    }
    .into()
}
