use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use crate::types::Paths;

#[derive(Debug, Copy, Clone)]
enum InputType {
  Pack,
  Unpack,
}

impl Default for InputType {
  fn default() -> InputType {
    InputType::Pack
  }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(protobuf_mapper), supports(struct_named))]
pub struct InputReceiver {
  #[darling(skip)]
  input_type: InputType,
  ident: syn::Ident,
  generics: syn::Generics,
  data: ast::Data<(), FieldReceiver>,
  message_type: Paths,
}

impl InputReceiver {
  pub fn to_unpack(self) -> Self {
    Self {
      input_type: InputType::Unpack,
      ..self
    }
  }
}

impl ToTokens for InputReceiver {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let InputReceiver {
      input_type,
      ref ident,
      ref generics,
      ref data,
      ref message_type,
    } = *self;

    let (imp, ty, wher) = generics.split_for_impl();
    let fields = data
      .as_ref()
      .take_struct()
      .expect("Should never be enum")
      .fields;

    match input_type {
      InputType::Pack => {
        let mut setter_lines: Vec<_> = vec![];

        let pack_lines: Vec<_> = fields
          .iter()
          .filter(|f| !f.skip_pack)
          .map(|f| {
            let field_ident = f.ident.as_ref().expect("field ident");
            let field_ty = &f.ty;
            let value_field_ident = if let Some(ident) = f.rename.as_ref() {
              ident
            } else {
              f.ident.as_ref().unwrap()
            };
            if let Some(map_fn) = f.map_fn.as_ref() {
              quote! {
                #value_field_ident: #map_fn(value.#field_ident),
              }
            } else {
              let value_expr = if f.proto_enum {
                let seter_ident = syn::Ident::new(&format!("set_{}", field_ident.to_string()) as &str, Span::call_site());
                setter_lines.push(quote! {
                  packed.#seter_ident(
                    <#field_ty as protobuf_mapper::ProtoEnum<_>>::into_proto_enum(value.#field_ident)
                  );
                });
                quote! { Default::default() }
              } else {
                quote! { value.#field_ident.pack()? }
              };
              quote! {
                #value_field_ident: #value_expr,
              }
            }
          })
          .collect();
        for message_type in &message_type.paths {
          let pack_block = quote! {
            {
              let mut packed = #message_type {
                #(#pack_lines)*
              };
              #(#setter_lines)*
              packed
            }
          };
          tokens.extend(quote! {
            impl #imp protobuf_mapper::ProtoPack<#message_type> for #ident #ty #wher {
              fn pack(self) -> protobuf_mapper::result::Result<#message_type> {
                let value = self;
                Ok(#pack_block)
              }
            }

            impl #imp protobuf_mapper::ProtoPack<Option<#message_type>> for #ident #ty #wher {
              fn pack(self) -> protobuf_mapper::result::Result<Option<#message_type>> {
                let value = self;
                Ok(Some(#pack_block))
              }
            }
          })
        }
      }
      InputType::Unpack => {
        let mut getter_lines: Vec<_> = vec![];
        let unpack_lines: Vec<_> = fields
          .iter()
          .map(|f| {
            let field_ident = &f.ident;
            let field_ty = &f.ty;
            let value_field_ident = if let Some(ident) = f.rename.as_ref() {
              ident
            } else {
              f.ident.as_ref().unwrap()
            };
            let field_expr = if let Some(map_fn) = f.map_fn.as_ref() {
              quote! {
                #map_fn(value.#value_field_ident)
              }
            } else {
              if f.proto_enum {
                getter_lines.push(quote! {
                  let #field_ident = <#field_ty as protobuf_mapper::ProtoEnum<_>>::unpack_enum(value.#value_field_ident());
                });
                quote! {
                  #field_ident
                }
              } else {
                quote! {
                  ProtoUnpack::unpack(value.#value_field_ident).map_err(|err| {
                    if let protobuf_mapper::result::Error::ValueNotPresent = err {
                      protobuf_mapper::result::Error::FieldValueNotPresent {
                        field_name: stringify!(#field_ident),
                      }
                    } else {
                      err
                    }
                  })?
                }
              }
            };
            quote! {
              #field_ident: #field_expr,
            }
          })
          .collect();

        for message_type in &message_type.paths {
          let unpack_block = quote! {
            #(#getter_lines)*
            Ok(#ident {
              #(#unpack_lines)*
            })
          };
          tokens.extend(quote! {
            impl #imp protobuf_mapper::ProtoUnpack<#message_type> for #ident #ty #wher {
              fn unpack(value: #message_type) -> protobuf_mapper::result::Result<#ident> {
                #unpack_block
              }
            }

            impl #imp protobuf_mapper::ProtoUnpack<Option<#message_type>> for #ident #ty #wher {
              fn unpack(value: Option<#message_type>) -> protobuf_mapper::result::Result<#ident> {
                if let Some(value) = value {
                  #unpack_block
                } else {
                  Err(protobuf_mapper::result::Error::ValueNotPresent)
                }
              }
            }
          })
        }
      }
    }
  }
}

#[derive(Debug, FromField)]
#[darling(attributes(protobuf_mapper))]
struct FieldReceiver {
  ident: Option<syn::Ident>,
  ty: syn::Type,
  #[darling(default)]
  rename: Option<syn::Ident>,
  #[darling(default)]
  map_fn: Option<syn::Path>,
  #[darling(default)]
  proto_enum: bool,
  #[darling(default)]
  skip_pack: bool,
}
