

use syn::{DeriveInput, DataStruct, Ident};
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
    "CJ",
];
}



pub fn instruction_transform(ast: &DeriveInput, name: &Ident) -> Result<proc_macro2::TokenStream, syn::parse::Error> {
    if let syn::Data::Struct(data) = &ast.data {
        let code_str = parse_code_attr(ast, "code")?;
        let code = parse_code_value(&code_str);
        let mask = parse_mask_value(&code_str);
        let format = parse_format_attr(ast)?;
        let decoder_ident = format_ident!("{}Decoder", name);
        let registery_ident = format_ident!("REGISTERY_{}", Ident::new(&name.to_string().to_uppercase(), name.span()));
        check_fields(data)?;
        Ok(quote!(
            bitfield_bitrange!{struct #name(InsnT)}
            insn_format!(#name, #format);
            impl #name {
                fn new(ir:InsnT) -> Instruction {
                    if (ir & #mask != #code) {
                        panic!(format!("ir 0x{:x} & mask 0x{:x} = 0x{:x}, expect 0x{:x}, it is not match code 0b{}!", ir, #mask, ir & #mask, #code, #code_str))
                    }
                    Instruction::new(#name(ir))
                }
                fn _ir(&self) ->  InsnT {
                    self.0
                }
            }
            impl InstructionImp for #name{}

            struct #decoder_ident;
            impl Decoder for #decoder_ident {
                fn code(&self) ->  InsnT {
                    #code
                }
                fn mask(&self) ->  InsnT {
                    #mask
                }
                fn matched(&self, ir:InsnT) -> bool {
                    ir & self.mask() == self.code()
                }
                fn decode(&self, ir:InsnT) -> Instruction {
                    #name::new(ir)
                }
            }

            #[distributed_slice(REGISTERY_INSN)]
            static #registery_ident: fn(&mut GlobalInsnMap) = |map| {map.registery(#decoder_ident)};
        ))
    } else {
        Err(syn::parse::Error::new(Span::call_site(), "Only Struct can derive"))
    }
}

fn check_fields(data: &DataStruct) -> Result<bool, syn::parse::Error> {
    if let syn::Fields::Unnamed(ref field) = data.fields {
        if field.unnamed.len() != 1 {
            return Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (InsnT)!"));
        }
        if let syn::Type::Path(ref path) = field.unnamed[0].ty {
            if path.path.segments.len() != 1 || path.path.segments[0].ident != Ident::new("InsnT", proc_macro2::Span::call_site()) {
                return Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (InsnT)!"));
            }
            return Ok(true);
        } else {
            return Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (InsnT)!"));
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), "expect struct \'name\' (InsnT)!"))
    }
}

fn parse_code_attr(ast: &DeriveInput, name: &str) -> Result<String, syn::parse::Error> {
    if let syn::NestedMeta::Lit(syn::Lit::Str(ref raw)) = parse_attr(ast, name)? {
        parse_raw_bits(&raw.value())
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as string with \"0b\" prefix!", name)))
    }
}

fn parse_raw_bits(code: &str) -> Result<String, syn::parse::Error> {
    lazy_static! {
        static ref VALID_CODE: Regex = Regex::new("^0b[10?_]+").unwrap();
        static ref VALID_BITS: Regex = Regex::new(&("^[10?]{1,".to_string() + &format!("{}", insn_len()) + "}")).unwrap();
        static ref BITS_REP: Regex = Regex::new("_|(?:0b)").unwrap();
    }
    if !VALID_CODE.is_match(code) {
        return Err(syn::parse::Error::new(Span::call_site(), "code contains invalid char, valid format is ^0b[1|0|?|_]+!"));
    }
    let bits = BITS_REP.replace_all(code, "");
    if !VALID_BITS.is_match(&bits) {
        return Err(syn::parse::Error::new(Span::call_site(), "code defined num of bits more than 32!"));
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

fn parse_format_attr(ast: &DeriveInput) -> Result<Ident, syn::parse::Error> {
    if let syn::NestedMeta::Meta(syn::Meta::Path(ref path)) = parse_attr(ast, "format")? {
        if let Some(ident) = path.get_ident() {
            if VALID_FORMAT_TYPE.contains(&&format!("{}", ident)[..]) {
                Ok(ident.clone())
            } else {
                Err(syn::parse::Error::new(Span::call_site(), format!("invalid \"{}\" value \"{}\", valid values are {:?}", "format", ident, *VALID_FORMAT_TYPE)))
            }
        } else {
            Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as Ident", "format")))
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected as Ident", "format")))
    }
}

fn parse_attr(ast: &DeriveInput, name: &str) -> Result<syn::NestedMeta, syn::parse::Error> {
    if let Some(attr) = ast.attrs.iter().find(|a| { a.path.segments.len() == 1 && a.path.segments[0].ident == name }) {
        if let syn::Meta::List(ref meta) = attr.parse_meta().unwrap() {
            if meta.nested.len() == 1 {
                Ok(meta.nested[0].clone())
            } else {
                Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected to be a single value", name)))
            }
        } else {
            Err(syn::parse::Error::new(Span::call_site(), format!("\"{}\" is expected to be a single value", name)))
        }
    } else {
        Err(syn::parse::Error::new(Span::call_site(), format!("attr \"{}\" missed", name)))
    }
}

