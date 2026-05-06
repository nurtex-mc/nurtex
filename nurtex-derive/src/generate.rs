use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

use crate::{extract_option_inner_type, extract_packet_field_feature};

/// Функция генерации кода для чтения значения с учётом особенности типа
fn generate_read_value(ty: &Type, feature: Option<String>) -> proc_macro2::TokenStream {
  if let Some(feat) = feature {
    match feat.as_str() {
      "varint" => quote! { <i32 as nurtex_codec::types::variable::VarI32>::read_var(buffer)? },
      "varlong" => quote! { <i64 as nurtex_codec::types::variable::VarI64>::read_var(buffer)? },
      "vec_end" => quote! {
        {
          let remaining = buffer.get_ref().len() - buffer.position() as usize;
          let mut vec = vec![0u8; remaining];

          for byte in &mut vec {
            *byte = u8::read_buf(buffer)?;
          }

          vec
        }
      },
      "vec_varint" => quote! {
        {
          let count = <i32 as nurtex_codec::types::variable::VarI32>::read_var(buffer)? as usize;
          let mut vec = Vec::with_capacity(count);

          for _ in 0..count {
            vec.push(<i32 as nurtex_codec::types::variable::VarI32>::read_var(buffer)?);
          }

          vec
        }
      },
      "vec_varlong" => quote! {
        {
          let count = <i32 as nurtex_codec::types::variable::VarI32>::read_var(buffer)? as usize;
          let mut vec = Vec::with_capacity(count);

          for _ in 0..count {
            vec.push(<i64 as nurtex_codec::types::variable::VarI64>::read_var(buffer)?);
          }

          vec
        }
      },
      _ => quote! { <#ty as nurtex_codec::Buffer>::read_buf(buffer)? },
    }
  } else {
    quote! { <#ty as nurtex_codec::Buffer>::read_buf(buffer)? }
  }
}

/// Функция генерации кода для записи значения с учётом особенности типа
fn generate_write_value(value: proc_macro2::TokenStream, feature: Option<String>) -> proc_macro2::TokenStream {
  if let Some(feat) = feature {
    match feat.as_str() {
      "varint" => quote! { <i32 as nurtex_codec::types::variable::VarI32>::write_var(&#value, buffer)?; },
      "varlong" => quote! { <i64 as nurtex_codec::types::variable::VarI64>::write_var(&#value, buffer)?; },
      "vec_end" => quote! {
        <i32 as nurtex_codec::types::variable::VarI32>::write_var(&(#value.len() as i32), buffer)?;

        for byte in &#value {
          byte.write_buf(buffer)?;
        }
      },
      "vec_varint" => quote! {
        <i32 as nurtex_codec::types::variable::VarI32>::write_var(&(#value.len() as i32), buffer)?;

        for item in &#value {
          <i32 as nurtex_codec::types::variable::VarI32>::write_var(item, buffer)?;
        }
      },
      "vec_varlong" => quote! {
        <i32 as nurtex_codec::types::variable::VarI32>::write_var(&(#value.len() as i32), buffer)?;
        for item in &#value {
          <i64 as nurtex_codec::types::variable::VarI64>::write_var(item, buffer)?;
        }
      },
      _ => quote! { #value.write_buf(buffer)?; },
    }
  } else {
    quote! { #value.write_buf(buffer)?; }
  }
}

/// Функция генерации чтения пакета
pub fn generate_read(input: &DeriveInput) -> proc_macro2::TokenStream {
  match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        let field_reads = fields.named.iter().map(|f| {
          let name = &f.ident;
          let ty = &f.ty;
          let feature = extract_packet_field_feature(f);

          if let Some(inner_ty) = extract_option_inner_type(ty) {
            let read_value = generate_read_value(&inner_ty, feature);
            quote! {
              #name: if <bool as nurtex_codec::Buffer>::read_buf(buffer)? {
                Some(#read_value)
              } else {
                None
              }
            }
          } else {
            let read_value = generate_read_value(ty, feature);
            quote! { #name: #read_value }
          }
        });

        quote! {
          Some(Self {
            #(#field_reads),*
          })
        }
      }
      Fields::Unit => quote! { Some(Self) },
      _ => quote! { compile_error!("packet derive only supports named fields") },
    },
    _ => quote! { compile_error!("packet derive only supports structs") },
  }
}

/// Функция генерации записи пакета
pub fn generate_write(input: &DeriveInput) -> proc_macro2::TokenStream {
  match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        let field_writes = fields.named.iter().map(|f| {
          let name = &f.ident;
          let ty = &f.ty;
          let feature = extract_packet_field_feature(f);

          if let Some(_) = extract_option_inner_type(ty) {
            let write_value = generate_write_value(quote! { val }, feature);
            quote! {
              <bool as nurtex_codec::Buffer>::write_buf(&self.#name.is_some(), buffer)?;
              if let Some(val) = &self.#name {
                #write_value
              }
            }
          } else {
            let write_value = generate_write_value(quote! { self.#name }, feature);
            quote! { #write_value }
          }
        });

        quote! {
          #(#field_writes)*
        }
      }
      Fields::Unit => quote! {},
      _ => quote! { compile_error!("packet derive only supports named fields") },
    },
    _ => quote! { compile_error!("packet derive only supports structs") },
  }
}
