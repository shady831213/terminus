use syn::parse::{Parse, ParseStream, Result, Error, ParseBuffer};
use syn::{parenthesized, braced, Ident, Token, LitInt};
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;
use std::collections::HashMap;

mod attr_kw {
    syn::custom_keyword!(fields);
    syn::custom_keyword!(fields32);
    syn::custom_keyword!(fields64);
}

#[derive(Debug)]
struct Csr {
    name: Ident,
    attrs: Punctuated<CsrAttr, Token![,]>,
}


impl Parse for Csr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content: ParseBuffer;
        braced!(content in input);
        Ok(Csr {
            name: name,
            attrs: content.parse_terminated(CsrAttr::parse)?,
        })
    }
}

type AttrPunctuated = Punctuated<Field, Token![;]>;

#[derive(Debug)]
struct Attr<K> {
    key: K,
    attrs: AttrPunctuated,
}

impl<K> Attr<K> {
    fn new(key: K, attrs: AttrPunctuated) -> Attr<K> {
        Attr { key, attrs }
    }
}


#[derive(Debug)]
enum CsrAttr {
    Fields(Attr<attr_kw::fields>),
    Fields32(Attr<attr_kw::fields32>),
    Fields64(Attr<attr_kw::fields64>),
}

macro_rules! parse_attr {
    ( $stream: ident, $key: path, $rt: path) => {
        || {
            let span = $stream.span();
            $stream.parse::<$key>()?;
            let content;
            syn::braced !(content in $ stream);
            Ok($rt(Attr::new($key(span), content.parse_terminated( <Field>::parse)?)))
        }
    };
}

impl Parse for CsrAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attr_kw::fields) {
            parse_attr!(input, attr_kw::fields, CsrAttr::Fields)()
        } else if lookahead.peek(attr_kw::fields32) {
            parse_attr!(input, attr_kw::fields32, CsrAttr::Fields32)()
        } else if lookahead.peek(attr_kw::fields64) {
            parse_attr!(input, attr_kw::fields64, CsrAttr::Fields64)()
        } else {
            Err(lookahead.error())
        }
    }
}

mod field_kw {
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW);
}

#[derive(Debug)]
enum FieldPrivilege {
    RO(field_kw::RO),
    WO(field_kw::WO),
    RW(field_kw::RW),
}

#[derive(Debug)]
struct Field {
    name: Ident,
    msb: LitInt,
    lsb: LitInt,
    privilege: FieldPrivilege,
}

impl Field {
    fn range(&self) -> (usize, usize) {
        (self.msb.base10_parse().unwrap(), self.lsb.base10_parse().unwrap())
    }

    fn same_name(&self, rhs: &Self) -> bool {
        self.name.to_string() == rhs.name.to_string()
    }

    fn overlap(&self, rhs: &Self) -> bool {
        let ((msb, lsb), (rmsb, rlsb)) = (self.range(), rhs.range());
        msb >= rlsb && msb <= rmsb || lsb >= rlsb && lsb <= rmsb
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        use FieldPrivilege::*;
        let content: ParseBuffer;
        parenthesized!(content in input);
        let privilege = if content.peek(field_kw::RO) {
            content.parse::<field_kw::RO>()?;
            RO(field_kw::RO(content.span()))
        } else if content.peek(field_kw::WO) {
            content.parse::<field_kw::WO>()?;
            WO(field_kw::WO(content.span()))
        } else if content.peek(field_kw::RW) {
            content.parse::<field_kw::RW>()?;
            RW(field_kw::RW(content.span()))
        } else {
            return Err(Error::new(content.span(), "expect [RO|WR|WO]"));
        };
        input.parse::<Token![:]>()?;

        let msb: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let lsb: LitInt = input.parse()?;

        if msb.base10_parse::<usize>()? < lsb.base10_parse::<usize>()? {
            return Err(Error::new(msb.span(), format!("msb {} is smaller than lsb {} !", msb.to_string(), lsb.to_string())));
        }

        Ok(Field {
            name,
            msb,
            lsb,
            privilege,
        })
    }
}

macro_rules! get_attr {
    ($attrs: expr, $exp: path) => {
        || {
            let _attr = $attrs.iter().filter_map(|f| {
                if let $exp(a) = f {
                    Some(a)
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            if _attr.len() == 0 {
                Ok(None)
            } else if _attr.len() == 1 {
                Ok(Some(_attr[0]))
            } else {
                Err(Error::new(_attr[1].key.span, format!("{:?} is redefined!", _attr[1].key)))
            }

        }
    };
}

macro_rules! expand_call {
    ($exp:expr) => {
        match $exp {
            Ok(result) => result,
            Err(err) => return err.to_compile_error(),
        }
    };
}

struct Fields<'a> {
    size: usize,
    fields: Vec<&'a Field>,
}

impl<'a> Fields<'a> {
    fn new(size: usize) -> Self {
        Fields {
            size: size,
            fields: vec![],
        }
    }

    fn overflow(&self, field: &Field) -> bool {
        let (msb, lsb) = field.range();
        msb >= self.size || lsb >= self.size
    }

    fn add(&mut self, field: &'a Field) -> Result<()> {
        if self.overflow(field) {
            Err(Error::new(field.name.span(), format!("field {}{:?} overflow!", field.name.to_string(), field.range())))
        } else {
            for prev in self.fields.iter() {
                if field.same_name(prev) {
                    return Err(Error::new(field.name.span(), format!("field {} is redefined!", field.name.to_string())));
                }
                if field.overlap(prev) {
                    return Err(Error::new(field.name.span(), format!("field {}{:?} is overlapped with field {}{:?}!", field.name.to_string(), field.range(), prev.name.to_string(), prev.range())));
                }
            }
            Ok(self.fields.push(field))
        }
    }

    fn default_field(&self, id: &Ident) -> Field {
        Field {
            name: id.clone(),
            msb: LitInt::new(&format!("{}", self.size - 1), id.span()),
            lsb: LitInt::new("0", id.span()),
            privilege: FieldPrivilege::RW(field_kw::RW(id.span())),
        }
    }

    fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

struct Trait {
    name: Ident,
    field_names: HashMap<String, Ident>,
    trait_name: Ident,
}

impl Trait {
    fn new(name: Ident) -> Self {
        let trait_name = format_ident!("{}Trait", name.to_string());
        Trait { name, field_names: HashMap::new(), trait_name }
    }

    fn add(&mut self, field: &Field) {
        self.field_names.insert(field.name.to_string(), field.name.clone());
    }

    fn gen(&self) -> TokenStream {
        let fns = self.field_names.values()
            .map(|name| {
                let setter = format_ident!("set_{}", name);
                let getter_msg = format!("{} not implement {} in current xlen setting!", self.name.to_string(), name.to_string());
                let setter_msg = format!("{} not implement {} in current xlen setting!", self.name.to_string(), setter.to_string());
                quote! {
                fn #name(&self) -> RegT { panic!(#getter_msg)}
                fn #setter(&self, value:RegT) { panic!(#setter_msg)}
            }
            })
            .fold(quote! {}, |acc, q| {
                quote! {
                #acc
                #q
                }
            });
        let trait_name = &self.trait_name;
        quote! {
            trait #trait_name {
                #fns
            }
        }
    }
}


pub fn expand(input: TokenStream) -> TokenStream {
    let csr: Csr = expand_call!(syn::parse2(input));
    let fields = expand_call!(get_attr!(csr.attrs, CsrAttr::Fields)());
    let fields32 = expand_call!(get_attr!(csr.attrs, CsrAttr::Fields32)());
    let fields64 = expand_call!(get_attr!(csr.attrs, CsrAttr::Fields64)());

    let mut field32s = Fields::new(32);
    let mut field64s = Fields::new(64);
    let mut csr_trait = Trait::new(csr.name.clone());
    if let Some(Attr { key: _, attrs }) = fields {
        for field in attrs {
            expand_call!(field32s.add(field));
            expand_call!(field64s.add(field));
            csr_trait.add(field);
        }
    }
    if let Some(Attr { key: _, attrs }) = fields32 {
        for field in attrs {
            expand_call!(field32s.add(field));
            csr_trait.add(field);
        }
    }
    if let Some(Attr { key: _, attrs }) = fields64 {
        for field in attrs {
            expand_call!(field64s.add(field));
            csr_trait.add(field);
        }
    }
    let default_id = Ident::new(&csr.name.to_string().to_lowercase(), csr.name.span());
    if field32s.is_empty() {
        let defalut_field = field32s.default_field(&default_id);
        field32s.add(&defalut_field);
        csr_trait.add(&defalut_field);
    }
    if field64s.is_empty() {
        let defalut_field = field64s.default_field(&default_id);
        field64s.add(&defalut_field);
        csr_trait.add(&defalut_field);
    }

    csr_trait.gen()
}




