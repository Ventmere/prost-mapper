use darling::{ast, FromDeriveInput, FromVariant};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::types::Paths;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(protobuf_mapper), supports(enum_unit))]
pub struct InputReceiver {
  ident: syn::Ident,
  generics: syn::Generics,
  data: ast::Data<VariantReceiver, ()>,
  proto_enum_type: Paths,
}

impl ToTokens for InputReceiver {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let InputReceiver {
      ref ident,
      ref generics,
      ref data,
      ref proto_enum_type,
    } = *self;

    let (imp, ty, wher) = generics.split_for_impl();
    let variants = data.as_ref().take_enum().expect("Should never be struct");

    let names: Vec<_> = variants
      .iter()
      .map(|v| {
        let v_ident = &v.ident;
        let v_name = v.ident.to_string();
        quote! {
          Self::#v_ident => #v_name,
        }
      })
      .collect();

    let name = ident.to_string();

    tokens.extend(quote! {
      impl #imp protobuf_mapper::ProtoEnumMeta for #ident #ty #wher {
        const NAME: &'static str = #name;

        fn get_variant_name(&self) -> &'static str {
          match *self {
            #(#names)*
          }
        }
      }
    });

    if proto_enum_type.paths.len() == 1 {
      let proto_enum_type = &proto_enum_type.paths[0];

      tokens.extend(quote! {
        impl #imp protobuf_mapper::ProtoPack<i32> for #ident #ty #wher
        {
          fn pack(self) -> protobuf_mapper::result::Result<i32> {
            let v = <Self as protobuf_mapper::ProtoEnum<#proto_enum_type>>::into_proto_enum(self);
            Ok(v.into())
          }
        }

        impl #imp protobuf_mapper::ProtoUnpack<i32> for #ident #ty #wher
        {
          fn unpack(v: i32) -> protobuf_mapper::result::Result<Self> {
            <Self as protobuf_mapper::ProtoEnum<#proto_enum_type>>::unpack_i32(v)
          }
        }
      });
    }

    for proto_enum_type in &proto_enum_type.paths {
      let (s2p, p2s): (Vec<_>, Vec<_>) = variants
        .iter()
        .map(|v| {
          let v_ident = &v.ident;
          let proto_ident = if let Some(ref ident) = v.rename {
            ident
          } else {
            v_ident
          };
          (
            quote! {
              Self::#v_ident => #proto_enum_type::#proto_ident,
            },
            quote! {
              #proto_enum_type::#proto_ident => Self::#v_ident,
            },
          )
        })
        .unzip();

      tokens.extend(quote! {
        impl #imp protobuf_mapper::ProtoEnum<#proto_enum_type> for #ident #ty  #wher {
          fn from_i32(v: i32) -> Option<Self> {
            #proto_enum_type::from_i32(v)
              .map(|p| {
                match p {
                  #(#p2s)*
                }
              })
          }

          fn into_proto_enum(self) -> #proto_enum_type {
            match self {
              #(#s2p)*
            }
          }

          fn unpack_enum(v: #proto_enum_type) -> Self {
            match v {
              #(#p2s)*
            }
          }
        }
      })
    }
  }
}

#[derive(Debug, FromVariant)]
#[darling(attributes(protobuf_mapper))]
struct VariantReceiver {
  ident: syn::Ident,
  #[darling(default)]
  rename: Option<syn::Ident>,
}
