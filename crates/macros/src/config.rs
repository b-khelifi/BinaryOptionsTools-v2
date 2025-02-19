use proc_macro2::TokenStream as TokenStream2;

use darling::{ast, util, FromDeriveInput, FromField};
use quote::{quote, ToTokens};
use syn::{Generics, Ident, Type};


#[derive(Debug, FromField)]
#[darling(attributes(lorem))]
struct ConfigField {
    ident: Option<Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lorem), supports(struct_named))]
pub struct Config {
    ident: Ident,
    data: ast::Data<util::Ignored, ConfigField>,
    generics: Generics
}

impl ToTokens for Config {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fields = &self.data.as_ref().take_struct().expect("Only available for structs");
        let name = &self.ident;
        let new_name = match format!("{}", name) {
            n if n.starts_with("_") => Ident::new(&n[1..], name.span()),
            n => Ident::new(&format!("{}Config", n), name.span())
        };
        let inner_name = Ident::new(&format!("{}Inner", new_name), new_name.span());
        let builder_name = Ident::new(&format!("{}Builder", new_name), new_name.span());

        let fields_iter = fields.iter().map(|f| f.field());
        let fields_builders = fields.iter().map(|f| f.builder());
        let fn_iter = fields.iter();
        let field_names = fields.iter().filter_map(|f| f.ident.as_ref());
        let field_names2 = field_names.clone();
        let field_names3 = field_names.clone();
        let field_names4 = field_names.clone();
        let field_names5 = field_names.clone();
        let field_none = fields.iter().map(|f| f.field_none());
        let field_names_string = field_names.clone().map(|f| format!("{}", &f));
        let field_type = fields.iter().map(|f| &f.ty);
        let generics = &self.generics;
        
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        tokens.extend(quote! {
            #[derive(Clone)]
            pub struct #new_name #generics {
                inner: ::std::sync::Arc<::std::sync::Mutex<#inner_name #ty_generics>> 
            }

            pub struct #inner_name #generics {
                #(#fields_iter),*
            }

            pub struct #builder_name #generics {
                #(#field_names: ::std::option::Option<#field_type>),*
            }

            impl #impl_generics #name #ty_generics #where_clause {
                pub fn builder(self) -> #builder_name #ty_generics {
                    #builder_name::from(self)
                }
            }


            impl #impl_generics #new_name #ty_generics #where_clause {
                #(#fn_iter)*
            }

            impl #impl_generics #builder_name #ty_generics #where_clause {
                #(#fields_builders)*

                pub fn new() -> #builder_name #ty_generics {
                    Self {
                        #(#field_none),*
                    }
                }

                pub fn build(self) -> ::std::result::Result<#new_name #ty_generics, ::std::boxed::Box<dyn ::core::error::Error + ::core::marker::Send + ::core::marker::Sync>> {
                    #new_name::try_from(self)
                }
            }

            impl #impl_generics From<#name #ty_generics> for #new_name #ty_generics #where_clause{
                fn from(value: #name #ty_generics) -> Self {
                    let inner = #inner_name {
                        #(#field_names2: value.#field_names2),*
                    };
                    Self {
                        inner: ::std::sync::Arc::new(::std::sync::Mutex::new(inner))
                    }
                }
            }

            impl #impl_generics From<#name #ty_generics> for #builder_name #ty_generics #where_clause {
                fn from(value: #name #ty_generics) -> Self {
                    Self {
                        #(#field_names3: ::std::option::Option::Some(value.#field_names3)),*
                    }
                }
            }

            impl #impl_generics TryFrom<#new_name #ty_generics> for #name #ty_generics #where_clause{
                type Error = ::anyhow::Error;

                fn try_from(value: #new_name #ty_generics) -> ::std::result::Result<Self, Self::Error> {
                    let inner = value.inner.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?;
                    Ok(
                        Self {
                            #(#field_names4: inner.#field_names4.clone()),*
                        }
                    )
                }
            }

            impl #impl_generics TryFrom<#builder_name #ty_generics> for #new_name #ty_generics #where_clause{
                type Error = ::std::boxed::Box<dyn ::core::error::Error + ::core::marker::Send + ::core::marker::Sync>;

                fn try_from(value: #builder_name #ty_generics) -> ::std::result::Result<Self, Self::Error> {
                    let inner = #inner_name {
                        #(#field_names5: value.#field_names5.ok_or(::std::io::Error::new(::std::io::ErrorKind::InvalidInput, format!("Option for field '{}' was None", #field_names_string)))?),*
                    };
                    Ok(
                        Self {
                            inner: ::std::sync::Arc::new(::std::sync::Mutex::new(inner))
                        }
                    )
                }
            }
        });
    }
}
impl ToTokens for ConfigField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = self.ident.as_ref().expect("Only fields with intent allowed");
        let dtype = &self.ty;
        let set_name = Ident::new(&format!("set_{}", name), name.span());
        let get_name = Ident::new(&format!("get_{}", name), name.span());
        tokens.extend(quote! {
            pub fn #set_name(&self, value: #dtype) ->  ::anyhow::Result<()> {
                let mut inner = self.inner.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?;
                inner.#name = value;
                Ok(())
            }

            pub fn #get_name(&self) -> ::anyhow::Result<#dtype> {
                Ok(self.inner.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?.#name.clone())
            }
        });
    }
}

impl ConfigField {
    fn field(&self) -> TokenStream2 {
        let name = self.ident.as_ref().expect("should have a name");
        let dtype = &self.ty;
        quote! {
            #name: #dtype
        }
    }

    fn builder(&self) -> TokenStream2 {
        let name = self.ident.as_ref().expect("should have a name");
        let dtype = &self.ty;
        quote! {
            pub fn #name(mut self, value: #dtype) -> Self {
                self.#name = Some(value);
                self
            }
        }
    }

    fn field_none(&self) -> TokenStream2 {
        let name = self.ident.as_ref().expect("should have a name");
        let dtype = &self.ty;
        quote! {
            #name: ::std::option::Option::None::<#dtype>
        }
    }
}