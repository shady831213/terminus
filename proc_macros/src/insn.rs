use syn::{DeriveInput, DataStruct, Ident, Result, NestedMeta, LitStr};
use syn::parse::Error;
use proc_macro2::Span;
use regex::Regex;
use terminus_global::{InsnT, insn_len};

lazy_static! {
static ref VALID_FORMAT_TYPE:Vec<&'static str> = vec![
    "USER_DEFINE",
    "R",
    "I",
    "S",
    "B",
    "U",
    "J",
    "CR",
    "CIW",
    "CI",
    "CSS",
    "CL",
    "CS",
    "CB",
    "CA",
    "CJ",
];
}



pub fn expand(ast: &DeriveInput, name: &Ident) -> Result<proc_macro2::TokenStream> {
    if let syn::Data::Struct(data) = &ast.data {
        let code_str = parse_code_attr(ast, "code")?;
        let code = parse_code_value(&code_str);
        let mask = parse_mask_value(&code_str);
        let format = parse_format_attr(ast)?;
        let decoder_ident = format_ident!("{}Decoder", name);
        let registery_ident = format_ident!("REGISTERY_{}", Ident::new(&name.to_string().to_uppercase(), name.span()));
        let name_string = name.to_string();
        check_fields(data, name)?;
        Ok(quote!(
            impl BitRange<InsnT> for #name {
                fn bit_range(&self, msb: usize, lsb: usize) -> InsnT {
                    let width = msb - lsb + 1;
                    if width == (std::mem::size_of::<InsnT>() << 3) {
                        self.0
                    } else {
                        let mask:InsnT = ((1 as InsnT) << (width as InsnT)) - 1;
                        ((self.0 >> (lsb as InsnT)) & mask)
                    }
                }

                fn set_bit_range(&mut self, msb: usize, lsb: usize, value: InsnT) {
                    let width = msb - lsb + 1;
                    let bitlen = (std::mem::size_of::<InsnT>() << 3);
                    if width == bitlen {
                        self.0 = value
                    } else {
                        let low = self.0 & (((1 as InsnT) << (lsb as InsnT)) - 1);
                        let high = if msb == bitlen - 1 {0} else {(self.0 >> ((msb + 1) as InsnT)) << ((msb + 1) as InsnT)};
                        let mask:InsnT = ((1 as InsnT) << (width as InsnT)) - 1;
                        self.0 = high | low | (((value as InsnT) & mask) << (lsb as InsnT));
                    }
                }
            }
            insn_format!(#name, #format);
            impl #name {
                fn new(ir:InsnT) -> Instruction {
                    if (ir & #mask != #code) {
                        panic!(format!("ir 0x{:x} & mask 0x{:x} = 0x{:x}, expect 0x{:x}, it is not match code 0b{}!", ir, #mask, ir & #mask, #code, #code_str))
                    }
                    Instruction::new(#name(ir))
                }
                #[inline(always)]
                fn _ir(&self) ->  InsnT {
                    self.0
                }
            }
            impl InsnClone for #name{
                fn clone(&self) -> Instruction {
                    #name::new(self.0)
                }
            }
            impl InstructionImp for #name{}

            struct #decoder_ident;
            impl Decoder for #decoder_ident {
                #[inline(always)]
                fn code(&self) ->  InsnT {
                    #code
                }
                #[inline(always)]
                fn mask(&self) ->  InsnT {
                    #mask
                }
                #[inline(always)]
                fn matched(&self, ir:InsnT) -> bool {
                    ir & self.mask() == self.code()
                }
                #[inline(always)]
                fn decode(&self, ir:InsnT) -> Instruction {
                    #name::new(ir)
                }
                #[inline(always)]
                fn name(&self) -> String{
                    #name_string.to_string()
                }
            }

            #[distributed_slice(REGISTERY_INSN)]
            static #registery_ident: fn(&mut GlobalInsnMap) = |map| {map.registery(#decoder_ident)};
        ))
    } else {
        Err(Error::new(name.span(), "Only Struct can derive"))
    }
}

fn check_fields(data: &DataStruct, name: &Ident) -> Result<bool> {
    let msg = format!("expect \'struct {}(InsnT);\' !", name.to_string());
    if let syn::Fields::Unnamed(ref field) = data.fields {
        if field.unnamed.len() != 1 {
            return Err(Error::new(field.paren_token.span, msg));
        }
        if let syn::Type::Path(ref path) = field.unnamed[0].ty {
            if path.path.segments.len() != 1 || path.path.segments[0].ident != Ident::new("InsnT", Span::call_site()) {
                return Err(Error::new(path.path.segments[0].ident.span(), msg));
            }
            return Ok(true);
        } else {
            return Err(Error::new(name.span(), msg));
        }
    } else {
        Err(Error::new(name.span(), msg))
    }
}

fn parse_code_attr(ast: &DeriveInput, name: &str) -> Result<String> {
    let Attr { ident, attr } = parse_attr(ast, name)?;
    if let NestedMeta::Lit(syn::Lit::Str(ref raw)) = attr {
        parse_raw_bits(raw)
    } else {
        Err(Error::new(ident.span(), format!("\"{}\" is expected as string with \"0b\" prefix!", name)))
    }
}

fn parse_raw_bits(lit: &LitStr) -> Result<String> {
    let code = lit.value();
    lazy_static! {
        static ref VALID_CODE: Regex = Regex::new("^0b[10?_]+$").unwrap();
        static ref VALID_BITS: Regex = Regex::new(&("^[10?]{1,".to_string() + &format!("{}", insn_len()) + "}$")).unwrap();
        static ref BITS_REP: Regex = Regex::new("_|(?:0b)").unwrap();
    }
    if !VALID_CODE.is_match(&code) {
        return Err(Error::new(lit.span(), "code contains invalid char, valid format is ^0b[1|0|?|_]+!"));
    }
    let bits = BITS_REP.replace_all(&code, "");
    if !VALID_BITS.is_match(&bits) {
        return Err(Error::new(lit.span(), format!("code defined num of bits more than {}!", insn_len())));
    }
    if bits.len() < insn_len() {
        Ok(ext_bits(&bits, insn_len()))
    } else {
        Ok(bits.to_string())
    }
}

fn ext_bits(bits: &str, cap: usize) -> String {
    if bits.len() == cap {
        bits.to_string()
    } else {
        ext_bits(&("?".to_owned() + bits), cap)
    }
}

fn parse_code_value(bits: &str) -> InsnT {
    lazy_static! {
        static ref QUE: Regex = Regex::new("[?]").unwrap();
    }
    InsnT::from_str_radix(&QUE.replace_all(bits, "0"), 2).unwrap()
}

fn parse_mask_value(bits: &str) -> InsnT {
    lazy_static! {
        static ref ZERO: Regex = Regex::new("0").unwrap();
    }
    parse_code_value(&ZERO.replace_all(bits, "1"))
}

fn parse_format_attr(ast: &DeriveInput) -> Result<Ident> {
    let Attr { ident, attr } = parse_attr(ast, "format")?;
    if let NestedMeta::Meta(syn::Meta::Path(ref path)) = attr {
        if let Some(ident) = path.get_ident() {
            if VALID_FORMAT_TYPE.contains(&&format!("{}", ident)[..]) {
                Ok(ident.clone())
            } else {
                Err(Error::new(ident.span(), format!("invalid \"{}\" value \"{}\", valid values are {:?}", "format", ident, *VALID_FORMAT_TYPE)))
            }
        } else {
            Err(Error::new(ident.span(), format!("\"{}\" is expected as Ident", "format")))
        }
    } else {
        Err(Error::new(ident.span(), format!("\"{}\" is expected as Ident", "format")))
    }
}

struct Attr {
    ident: Ident,
    attr: NestedMeta,
}

impl Attr {
    fn new(ident: Ident, attr: NestedMeta) -> Self {
        Attr { ident, attr }
    }
}

fn parse_attr(ast: &DeriveInput, name: &str) -> Result<Attr> {
    if let Some(attr) = ast.attrs.iter().find(|a| { a.path.segments.len() == 1 && a.path.segments[0].ident == name }) {
        let meta = attr.parse_meta()?;
        if let syn::Meta::List(ref nested_meta) = meta {
            if nested_meta.nested.len() == 1 {
                Ok(Attr::new(attr.path.segments[0].ident.clone(), nested_meta.nested[0].clone()))
            } else {
                Err(Error::new(attr.path.segments[0].ident.span(), format!("\"{}\" is expected to be a single value", name)))
            }
        } else {
            Err(Error::new(attr.path.segments[0].ident.span(), format!("\"{}\" is expected to be a single value", name)))
        }
    } else {
        Err(Error::new(Span::call_site(), format!("attr \"{}\" missed", name)))
    }
}

