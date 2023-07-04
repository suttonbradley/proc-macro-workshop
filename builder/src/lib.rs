use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    println!("Input AST is:\n{ast:#?}\n");

    // Get struct identifier and form builder identifier
    let struct_ident = &ast.ident;
    let builder_name = format!("{}Builder", struct_ident);
    let builder_ident = syn::Ident::new(builder_name.as_str(), struct_ident.span());

    // Get struct fields for iteration within quote macro
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("Builder only supported for structs");
    };
    // Pull identifier and type of each field and make into a std::option::Option w/ the same identifier
    let fields_optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: std::option::Option<#ty>}
    });

    // Methods on builder struct
    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    // Lines that create a struct from a builder
    let field_lines_build = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
        }
    });
    // Lines that specify each builder field as None (to start)
    let field_lines_none = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

    // Rust code that is produced and returned (# will be filled with vars from Rust code)
    quote!(
        pub struct #builder_ident {
            #(#fields_optionized),*
        }

        impl #builder_ident {
            #(#methods)*

            // Note: full paths here because you can't guarantee where the user USES the macro that (e.g.) std::error will be in scope
            fn build(&self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
                // TODO: make this a walk of the fields in the struct
                Ok(#struct_ident {
                    #(#field_lines_build),*
                })
            }
        }

        impl #struct_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#field_lines_none),*
                }
            }
        }
    )
    .into()
}
