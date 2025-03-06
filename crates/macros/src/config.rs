use proc_macro2::TokenStream as TokenStream2;

use darling::{ast, util, FromDeriveInput, FromField, FromMeta};
use quote::{quote, ToTokens};
use syn::{Generics, Ident, Type};

#[derive(Debug, FromMeta)]
enum FieldConfig {
    #[darling(rename = "optional")]
    Optional,
    #[darling(rename = "iterator")]
    Iterator { dtype: Type, add_fn: Option<String> },
}

#[derive(Debug, FromField)]
#[darling(attributes(config))]
struct ConfigField {
    ident: Option<Ident>,
    ty: Type,
    extra: Option<FieldConfig>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(config), supports(struct_named))]
pub struct Config {
    ident: Ident,
    data: ast::Data<util::Ignored, ConfigField>,
    generics: Generics,
}

impl ToTokens for Config {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fields = &self
            .data
            .as_ref()
            .take_struct()
            .expect("Only available for structs");
        let name = &self.ident;
        let new_name = match format!("{}", name) {
            n if n.starts_with("_") => Ident::new(&n[1..], name.span()),
            n => Ident::new(&format!("{}Config", n), name.span()),
        };
        let builder_name = Ident::new(&format!("{}Builder", new_name), new_name.span());

        let fields_builders = fields.iter().map(|f| f.builder());
        let fn_iter = fields.iter();
        let field_names = fields.iter().filter_map(|f| f.ident.as_ref());
        let field_names2 = field_names.clone();
        let field_names3 = field_names.clone();
        let field_names4 = field_names.clone();
        let field_names5 = field_names.clone();
        let ok_or_error = fields.iter().map(|f| f.ok_panic_default());
        let field_none = fields.iter().map(|f| f.field_none());
        let field_type = fields.iter().map(|f| &f.ty);
        let field_type2 = field_type.clone();
        let generics = &self.generics;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        tokens.extend(quote! {
            #[derive(Clone)]
            pub struct #new_name #generics {
                #(#field_names: ::std::sync::Arc<::std::sync::Mutex<#field_type>>),*
            }

            pub struct #builder_name #generics {
                #(#field_names2: ::std::option::Option<#field_type2>),*
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

                pub fn build(self) -> ::anyhow::Result<#new_name #ty_generics> {
                    #new_name::try_from(self)
                }
            }

            impl #impl_generics ::std::default::Default for #builder_name #ty_generics #where_clause {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl #impl_generics From<#name #ty_generics> for #new_name #ty_generics #where_clause {
                fn from(value: #name #ty_generics) -> Self {
                    Self {
                        #(#field_names3: ::std::sync::Arc::new(::std::sync::Mutex::new(value.#field_names3))),*
                    }
                }
            }

            impl #impl_generics From<#name #ty_generics> for #builder_name #ty_generics #where_clause {
                fn from(value: #name #ty_generics) -> Self {
                    Self {
                        #(#field_names4: ::std::option::Option::Some(value.#field_names4)),*
                    }
                }
            }

            impl #impl_generics TryFrom<#new_name #ty_generics> for #name #ty_generics #where_clause {
                type Error = ::anyhow::Error;

                fn try_from(value: #new_name #ty_generics) -> ::std::result::Result<Self, Self::Error> {
                    Ok(
                        Self {
                            #(#field_names5: value.#field_names5.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?.clone()),*
                        }
                    )
                }
            }

            impl #impl_generics TryFrom<#builder_name #ty_generics> for #new_name #ty_generics #where_clause {
                type Error = ::anyhow::Error;

                fn try_from(value: #builder_name #ty_generics) -> ::std::result::Result<Self, Self::Error> {
                    Ok(
                        Self {
                            #(#ok_or_error),*
                        }
                    )
                }
            }
        });
    }
}

impl ToTokens for ConfigField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = self.ident.as_ref().expect("Only fields with ident allowed");
        let dtype = &self.ty;
        let set_name = Ident::new(&format!("set_{}", name), name.span());
        let get_name = Ident::new(&format!("get_{}", name), name.span());
        let extra = if let Some(FieldConfig::Iterator { dtype, add_fn }) = &self.extra {
            let add_name = Ident::new(&format!("add_{}", name), name.span());
            let add_fn = if let Some(add) = add_fn {
                Ident::new(add, name.span())
            } else {
                Ident::new("push", name.span())
            };
            quote! {
                pub fn #add_name(&self, value: #dtype) -> ::anyhow::Result<()> {
                    let mut field = self.#name.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?;
                    field.#add_fn(value);
                    Ok(())

                }
            }
        } else {
            quote! {}
        };
        tokens.extend(quote! {
            #extra

            pub fn #set_name(&self, value: #dtype) -> ::anyhow::Result<()> {
                let mut field = self.#name.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?;
                *field = value;
                Ok(())
            }

            pub fn #get_name(&self) -> ::anyhow::Result<#dtype> {
                Ok(self.#name.lock().map_err(|e| ::anyhow::anyhow!("Poison error {e}"))?.clone())
            }
        });
    }
}

impl ConfigField {
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

    fn ok_panic_default(&self) -> TokenStream2 {
        let name = self.ident.as_ref().expect("should have a name");
        let name_str = format!("{}", name);
        if let Some(extra) = &self.extra {
            match extra {
                FieldConfig::Iterator { .. } => {
                    quote! {
                        #name: ::std::sync::Arc::new(::std::sync::Mutex::new(value.#name.unwrap_or_else(::std::default::Default::default)))
                    }
                }
                FieldConfig::Optional => {
                    quote! {
                        #name: ::std::sync::Arc::new(::std::sync::Mutex::new(value.#name.unwrap_or(::std::option::Option::None)))
                    }
                }
            }
        } else {
            quote! {
                #name: ::std::sync::Arc::new(::std::sync::Mutex::new(value.#name.ok_or(::anyhow::anyhow!("Option for field '{}' was None", #name_str))?))
            }
        }
    }
}
