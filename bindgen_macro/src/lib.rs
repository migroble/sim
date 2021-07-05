#![deny(clippy::pedantic)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn bindgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let ident = input.ident.clone();
    TokenStream::from(quote! {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen::prelude::*;

        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        #input

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen]
        impl #ident {
            pub fn pin_count(&self) -> usize {
                Component::pin_count(self)
            }

            pub fn update(&mut self, mut io: JsIO) {
                unsafe {
                    Component::update(self, &mut io);
                }
            }
        }
    })
}

#[proc_macro_attribute]
pub fn constrgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let ident = input.ident.clone();
    TokenStream::from(quote! {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen::prelude::*;

        #[derive(Default)]
        #input

        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        impl #ident {
            #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
            pub fn new() -> Self {
                Self::default()
            }
        }
    })
}
