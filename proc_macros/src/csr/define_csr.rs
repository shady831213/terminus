use syn::parse::{Parse, ParseStream, Result, Error, ParseBuffer};
use syn::{parenthesized, braced, Ident, Token, LitInt};
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use super::*;

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

#[derive(Debug)]
struct Field {
    name: Ident,
    msb: LitInt,
    lsb: LitInt,
    privilege: CsrPrivilege,
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

    fn setter_name(&self) -> Ident {
        format_ident!("set_{}", self.name)
    }

    fn getter_name(&self) -> Ident {
        self.name.clone()
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        use CsrPrivilege::*;
        let content: ParseBuffer;
        parenthesized!(content in input);
        let privilege = if content.peek(privilege_kw::RO) {
            content.parse::<privilege_kw::RO>()?;
            RO(privilege_kw::RO(content.span()))
        } else if content.peek(privilege_kw::WO) {
            content.parse::<privilege_kw::WO>()?;
            WO(privilege_kw::WO(content.span()))
        } else if content.peek(privilege_kw::RW) {
            content.parse::<privilege_kw::RW>()?;
            RW(privilege_kw::RW(content.span()))
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
    name: Ident,
    size: usize,
    fields: Vec<&'a Field>,
}

impl<'a> Fields<'a> {
    fn new(name: Ident, size: usize) -> Self {
        Fields {
            name,
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
            privilege: CsrPrivilege::RW(privilege_kw::RW(id.span())),
        }
    }

    fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    fn struct_name(&self) -> Ident {
        format_ident!("{}{}", self.name.to_string(),self.size)
    }

    fn struct_expand(&self, trait_name: &Ident) -> TokenStream {
        let fields = self.fields.iter()
            .map(|field| {
                let (setter, getter) = (field.setter_name(), field.getter_name());
                let (msb, lsb) = (&field.msb, &field.lsb);
                quote! {
                    #getter, #setter: #msb, #lsb;
                }
            })
            .fold(quote! {}, |acc, q| {
                quote! {
                    #acc
                    #q
                }
            });
        let set = self.fields.iter()
            .filter(|field| {
                match field.privilege {
                    CsrPrivilege::RW(_) => true,
                    CsrPrivilege::WO(_) => true,
                    CsrPrivilege::RO(_) => false
                }
            })
            .map(|field| {
                let lsb = &field.lsb;
                let setter = field.setter_name();
                quote! {
                    self.#setter(value >> (#lsb as RegT));
                }
            })
            .fold(quote! {}, |acc, q| {
                quote! {
                    #acc
                    #q
                }
            });
        let get = self.fields.iter()
            .filter(|field| {
                match field.privilege {
                    CsrPrivilege::RW(_) => true,
                    CsrPrivilege::WO(_) => false,
                    CsrPrivilege::RO(_) => true
                }
            })
            .map(|field| {
                let lsb = &field.lsb;
                let getter = field.getter_name();
                quote! {
                    (self.#getter() << (#lsb as RegT))
                }
            })
            .fold(quote! {(0 as RegT)}, |acc, q| {
                quote! {
                    #acc | #q
                }
            });
        let struct_name = self.struct_name();
        let size = format_ident!("u{}", self.size);
        quote! {
            #[derive(Copy, Clone)]
            struct #struct_name(#size);
            bitfield_bitrange! {struct #struct_name(#size)}
            impl #trait_name for #struct_name {
                fn get(&self) -> RegT {
                   #get
                }
                fn set(&mut self, value:RegT) {
                    #set
                }
                bitfield_fields! {
                    RegT;
                    #fields
                }
            }
        }
    }
}

struct FieldSet<'a> {
    name: Ident,
    field_names: HashMap<String, &'a Field>,
}

impl<'a> FieldSet<'a> {
    fn new(name: Ident) -> Self {
        FieldSet { name, field_names: HashMap::new() }
    }

    fn add(&mut self, field: &'a Field) {
        self.field_names.insert(field.name.to_string(), field);
    }

    fn trait_name(&self) -> Ident {
        format_ident!("{}Trait", self.name.to_string())
    }

    fn trait_expand(&self) -> TokenStream {
        let fns = self.field_names.values()
            .map(|field| {
                let (setter, getter) = (field.setter_name(), field.getter_name());
                let getter_msg = format!("{} not implement {} in current xlen setting!", self.name.to_string(), getter.to_string());
                let setter_msg = format!("{} not implement {} in current xlen setting!", self.name.to_string(), setter.to_string());
                quote! {
                fn #getter(&self) -> RegT { panic!(#getter_msg)}
                fn #setter(&mut self, value:RegT) { panic!(#setter_msg)}
            }
            })
            .fold(quote! {}, |acc, q| {
                quote! {
                #acc
                #q
                }
            });
        let trait_name = self.trait_name();
        quote! {
            pub trait #trait_name {
                fn get(&self) -> RegT;
                fn set(&mut self, value:RegT);
                #fns
            }
        }
    }

    fn top_expand(&self, struct32_name: &Ident, struct64_name: &Ident) -> TokenStream {
        let union_name = format_ident!("{}Union", self.name.to_string());
        let union_target = quote! {
            union #union_name {
                x32: #struct32_name,
                x64: #struct64_name,
            }
        };

        let top_name = &self.name;
        let trait_name = self.trait_name();

        let fns = self.field_names.values()
            .map(|field| {
                let (setter, getter) = (field.setter_name(), field.getter_name());
                quote! {
                fn #getter(&self) -> RegT {
                    match self.xlen {
                        XLen::X64 => unsafe { self.csr.x64.#getter() },
                        XLen::X32 => unsafe { self.csr.x32.#getter() }
                    }
                }
                fn #setter(&mut self, value:RegT) {
                    match self.xlen {
                        XLen::X64 => unsafe { self.csr.x64.#setter(value) },
                        XLen::X32 => unsafe { self.csr.x32.#setter(value) }
                    }
                }
            }
            })
            .fold(quote! {}, |acc, q| {
                quote! {
                #acc
                #q
                }
            });

        quote! {
            #union_target
            pub struct #top_name {
                xlen:XLen,
                csr:#union_name,
            }

            impl #top_name {
                pub fn new(xlen:XLen) -> #top_name {
                    #top_name{
                        xlen,
                        csr:#union_name{x64:{#struct64_name(0)}}
                    }
                }
            }

            impl #trait_name for #top_name {
                fn get(&self) -> RegT {
                    match self.xlen {
                        XLen::X64 => unsafe { self.csr.x64.get() },
                        XLen::X32 => unsafe { self.csr.x32.get() }
                    }
                }
                fn set(&mut self, value:RegT) {
                    match self.xlen {
                        XLen::X64 => unsafe { self.csr.x64.set(value) },
                        XLen::X32 => unsafe { self.csr.x32.set(value) }
                    }
                }
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

    let mut field32s = Fields::new(csr.name.clone(), 32);
    let mut field64s = Fields::new(csr.name.clone(), 64);
    let mut field_set = FieldSet::new(csr.name.clone());
    if let Some(Attr { key: _, attrs }) = fields {
        for field in attrs {
            expand_call!(field32s.add(field));
            expand_call!(field64s.add(field));
            field_set.add(field);
        }
    }
    if let Some(Attr { key: _, attrs }) = fields32 {
        for field in attrs {
            expand_call!(field32s.add(field));
            field_set.add(field);
        }
    }
    if let Some(Attr { key: _, attrs }) = fields64 {
        for field in attrs {
            expand_call!(field64s.add(field));
            field_set.add(field);
        }
    }
    let default_id = Ident::new(&csr.name.to_string().to_lowercase(), csr.name.span());
    let defalut_field32 = field32s.default_field(&default_id);
    let defalut_field64 = field64s.default_field(&default_id);
    if field32s.is_empty() {
        field32s.add(&defalut_field32);
        field_set.add(&defalut_field32);
    }
    if field64s.is_empty() {
        field64s.add(&defalut_field64);
        field_set.add(&defalut_field64);
    }

    let trait_name = field_set.trait_name();
    let trait_target = field_set.trait_expand();
    let struct32_target = field32s.struct_expand(&trait_name);
    let struct64_target = field64s.struct_expand(&trait_name);
    let top_target = field_set.top_expand(&field32s.struct_name(), &field64s.struct_name());

    quote! {
        #trait_target
        #struct32_target
        #struct64_target
        #top_target
    }
}




