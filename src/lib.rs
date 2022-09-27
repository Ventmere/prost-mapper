mod convert;
pub mod result;

use crate::result::Error;

#[allow(unused_imports)]
#[macro_use]
extern crate protobuf_mapper_codegen;

pub use self::convert::Json;
pub use protobuf_mapper_codegen::*;

pub trait ProtoPack<T>
where
  Self: Sized,
{
  fn pack(self) -> Result<T, Error>;
}

pub trait ProtoUnpack<T>
where
  Self: Sized,
{
  fn unpack(value: T) -> Result<Self, Error>;
}

pub trait ProtoEnumMeta {
  const NAME: &'static str;
  fn get_variant_name(&self) -> &'static str;
}

pub trait ProtoEnum<T>
where
  Self: Sized,
{
  fn from_i32(v: i32) -> Option<Self>;
  fn into_proto_enum(self) -> T;
  fn unpack_i32(v: i32) -> Result<Self, Error> where Self: ProtoEnumMeta{
    Self::from_i32(v).ok_or_else(|| Error::EnumDiscriminantNotFound {
      enum_name: Self::NAME,
      discriminant: v
    })
  }
  fn unpack_enum(v: T) -> Self;
}

impl<T1, T2> ProtoPack<Option<T1>> for Option<T2>
where
  T2: ProtoPack<T1>,
{
  fn pack(self) -> Result<Option<T1>, Error> {
    if let Some(value) = self {
      Ok(Some(value.pack()?))
    } else {
      Ok(None)
    }
  }
}

impl<T1, T2> ProtoUnpack<Option<T1>> for Option<T2>
where
  T2: ProtoUnpack<T1>,
{
  fn unpack(value: Option<T1>) -> Result<Self, Error> {
    if let Some(value) = value {
      Ok(Some(T2::unpack(value)?))
    } else {
      Ok(None)
    }
  }
}
