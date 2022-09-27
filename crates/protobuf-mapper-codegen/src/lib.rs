extern crate proc_macro;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod types;
mod derive_struct;
mod derive_enum;

macro_rules! try_parse {
  ($e:expr) => {
    match $e {
      Ok(v) => v,
      Err(e) => return TokenStream::from(e.write_errors()),
    }
  };
}

#[proc_macro_derive(ProtoPack, attributes(protobuf_mapper))]
pub fn derive_pack(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let receiver = try_parse!(derive_struct::InputReceiver::from_derive_input(&input));
  TokenStream::from(quote!(#receiver))
}

#[proc_macro_derive(ProtoUnpack, attributes(protobuf_mapper))]
pub fn derive_unpack(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let receiver = try_parse!(derive_struct::InputReceiver::from_derive_input(&input)).to_unpack();
  TokenStream::from(quote!(#receiver))
}

#[proc_macro_derive(ProtoEnum, attributes(protobuf_mapper))]
pub fn derive_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let receiver = try_parse!(derive_enum::InputReceiver::from_derive_input(
    &input
  ));
  TokenStream::from(quote!(#receiver))
}
