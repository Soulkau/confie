use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenStream as TS};
use quote::quote;
use syn::{
    Ident, LitStr, Path, Token, braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

struct Section {
    name: Ident,
    ty: Path,
}
struct Input {
    file: LitStr,
    sections: Punctuated<Section, Token![,]>,
}

impl Parse for Section {
    fn parse(i: ParseStream) -> syn::Result<Self> {
        let name = i.parse()?;
        i.parse::<Token![:]>()?;
        Ok(Section {
            name,
            ty: i.parse()?,
        })
    }
}
impl Parse for Input {
    fn parse(i: ParseStream) -> syn::Result<Self> {
        let file = i.parse()?;
        i.parse::<Token![,]>()?;
        let body;
        braced!(body in i);
        Ok(Input {
            file,
            sections: body.parse_terminated(Section::parse, Token![,])?,
        })
    }
}

fn lit(v: &toml::Value) -> Option<TS> {
    Some(match v {
        toml::Value::String(s) => quote!(#s),
        toml::Value::Integer(i) => {
            let l = Literal::i64_unsuffixed(*i);
            quote!(#l)
        }
        toml::Value::Boolean(b) => quote!(#b),
        toml::Value::Float(f) => {
            let l = Literal::f64_unsuffixed(*f);
            quote!(#l)
        }
        toml::Value::Array(a) => {
            let items = a.iter().map(lit).collect::<Option<Vec<_>>>()?;
            quote!(&[#(#items),*])
        }
        _ => return None,
    })
}

#[proc_macro]
pub fn load(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand(Input { file, sections }: Input) -> syn::Result<TS> {
    let err = |msg: String| syn::Error::new(file.span(), msg);

    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| err("CARGO_MANIFEST_DIR not set".into()))?;
    let path = std::path::Path::new(&dir).join(file.value());
    let raw =
        std::fs::read_to_string(&path).map_err(|e| err(format!("{}: {e}", path.display())))?;
    let doc: toml::Table = raw
        .parse()
        .map_err(|e| err(format!("{}: {e}", path.display())))?;

    let mut items = Vec::new();
    for Section { name, ty } in sections {
        let tbl = doc
            .get(&name.to_string())
            .and_then(|v| v.as_table())
            .ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    format!("no [{name}] table in {}", file.value()),
                )
            })?;

        let mut fields = Vec::new();
        for (k, v) in tbl {
            let key = syn::parse_str::<Ident>(k).map_err(|_| err(format!("bad key `{k}`")))?;
            let val = lit(v).ok_or_else(|| err(format!("unsupported value for `{k}`")))?;
            fields.push(quote!(#key: #val));
        }
        let konst = Ident::new(&name.to_string().to_uppercase(), name.span());
        items.push(quote!(pub const #konst: #ty = #ty { #(#fields),* };));
    }

    let p = path
        .to_str()
        .ok_or_else(|| err("path is not valid UTF-8".into()))?;
    Ok(quote! {
        const _: &str = include_str!(#p);
        #(#items)*
    })
}
