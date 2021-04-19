use syn::{parse_macro_input, DeriveInput, Data, Fields, DataEnum};
use proc_macro::{TokenStream};
use proc_macro2::Ident;
use quote::quote;

fn create_enum_elems(enum_name:&Ident, data_enum:&DataEnum)  -> Vec<proc_macro2::TokenStream> {
    data_enum.variants.iter().enumerate().map(| (idx,variant)| {
        let variant_ident = &variant.ident;

        match &variant.fields {
            Fields::Named(_) => {
                quote! { #enum_name::#variant_ident{ .. } =>  { #idx } }
            }

            Fields::Unnamed(fields) => {
                let fields_ident =
                    &fields.unnamed.iter().enumerate().map(|(idx,_)|{
                        proc_macro2::Ident::new(
                            &format!("x_{}",idx), proc_macro2::Span::call_site()
                        )
                    }).collect::<Vec<Ident>>();

                quote! { #enum_name::#variant_ident(  #(#fields_ident), * ) =>  { #idx } }
            }

            Fields::Unit => {
                quote! { #enum_name::#variant_ident =>  { #idx } }
            }
        }

    }).collect()
}


fn impl_identity(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    match &ast.data {
        Data::Struct(_) => { unimplemented!() }

        Data::Enum(data_enum) => {
            let variant_quotes = create_enum_elems(name,data_enum);

            let gen = quote! {
                impl #name {
                    pub fn position(&self) -> usize {
                        match self {
                            #(#variant_quotes), *
                        }
                    }
                }
            };

            println!("GENERATATED: {}",gen);

            gen.into()
        }

        Data::Union(_) => { unimplemented!() }
    }

}

#[proc_macro_derive(NumIdentity)]
pub fn num_identity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_identity(&input)

}