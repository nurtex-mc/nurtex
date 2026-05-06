use syn::{GenericArgument, Meta, PathArguments, Type};

/// Функция извлечения особенности поля пакета
pub fn extract_packet_field_feature(f: &syn::Field) -> Option<String> {
  let mut feature = None;

  for attr in &f.attrs {
    match &attr.meta {
      Meta::Path(p) => {
        if let Some(ident) = p.get_ident() {
          feature = Some(ident.to_string());
        }
      }
      _ => {}
    }
  }

  feature
}

/// Функция извлечения ID пакета
pub fn extract_packet_id(variant: &syn::Variant) -> Option<u32> {
  variant.attrs.iter().find(|a| a.path().is_ident("id")).and_then(|a| {
    if let syn::Meta::NameValue(nv) = &a.meta {
      if let syn::Expr::Lit(expr_lit) = &nv.value {
        match &expr_lit.lit {
          syn::Lit::Int(lit_int) => {
            let s = lit_int.to_string();
            if s.starts_with("0x") {
              u32::from_str_radix(&s[2..], 16).ok()
            } else {
              lit_int.base10_parse::<u32>().ok()
            }
          }
          syn::Lit::Str(lit_str) => {
            let s = lit_str.value();
            if s.starts_with("0x") {
              u32::from_str_radix(&s[2..], 16).ok()
            } else {
              s.parse::<u32>().ok()
            }
          }
          _ => None,
        }
      } else {
        None
      }
    } else {
      None
    }
  })
}

/// Функция извлечения типа из `Option<T>`
pub fn extract_option_inner_type(ty: &Type) -> Option<Type> {
  if let Type::Path(type_path) = ty {
    if let Some(segment) = type_path.path.segments.last() {
      if segment.ident == "Option" {
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
          if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
            return Some(inner_ty.clone());
          }
        }
      }
    }
  }

  None
}
