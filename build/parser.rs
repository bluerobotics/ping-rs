use std::collections::HashMap;
use std::io::{Read, Write};

use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
    Deserialize, Serialize, Serializer,
};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote;

macro_rules! ident {
    ($a:expr) => {{
        quote::format_ident!("{}", $a)
    }};
}

#[derive(Debug)]
enum PayloadType {
    CHAR,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    VECTOR(Box<PayloadType>, Option<u64>),
}

impl PayloadType {
    pub fn from_json(value: &serde_json::Value) -> Self {
        // Hack to have a recursive lambda function
        struct Recursive<'a> {
            pub get_type: &'a dyn Fn(Recursive, &str) -> PayloadType,
        }

        let fun = Recursive {
            get_type: &|r, name: &str| match name {
                "char" => PayloadType::CHAR,
                "u8" | "uint8_t" => PayloadType::U8,
                "u16" | "uint16_t" => PayloadType::U16,
                "u32" | "uint32_t" => PayloadType::U32,
                "i8" | "int8_t" => PayloadType::I8,
                "i16" | "int16_t" => PayloadType::I16,
                "i32" | "int32_t" => PayloadType::I32,
                "float" => PayloadType::F32,
                "vector" => PayloadType::VECTOR(
                    Box::new((r.get_type)(
                        r,
                        value.pointer("/vector/datatype").unwrap().as_str().unwrap(),
                    )),
                    None,
                ),
                something => panic!("No available type: {:#?}", something),
            },
        };

        return (fun.get_type)(fun, value.get("type").unwrap().as_str().unwrap());
    }

    pub fn to_rust(&self) -> Ident {
        let typ = match self {
            PayloadType::CHAR => "char",
            PayloadType::U8 => "u8",
            PayloadType::U16 => "u16",
            PayloadType::U32 => "u32",
            PayloadType::I8 => "i8",
            PayloadType::I16 => "i16",
            PayloadType::I32 => "i32",
            PayloadType::F32 => "f32",
            PayloadType::VECTOR(payload_typ, size) => {
                "u8" //quote::format_ident!("{}_length: {}", payload.name, payload_typ.to_rust(payload))
            }
        };

        return quote::format_ident!("{}", typ);
    }
}

#[derive(Debug)]
struct Payload {
    name: String,
    description: Option<String>,
    typ: PayloadType,
    units: Option<String>,
}

impl Payload {
    pub fn from_json(value: &serde_json::Value) -> Self {
        let unwrap_string = |name| match value.get(name) {
            Some(unit) => Some(unit.as_str().unwrap().into()),
            None => None,
        };

        Payload {
            name: value.get("name").unwrap().as_str().unwrap().into(),
            description: unwrap_string("description"),
            typ: PayloadType::from_json(value),
            units: unwrap_string("units"),
        }
    }
}

#[derive(Debug)]
struct MessageDefinition {
    name: String,
    id: u16,
    description: String,
    payload: Vec<Payload>,
}

impl MessageDefinition {
    pub fn from_json(name: &String, value: &serde_json::Value) -> Self {
        MessageDefinition {
            name: name.clone(),
            id: value.get("id").unwrap().as_u64().unwrap() as u16,
            description: value.get("description").unwrap().as_str().unwrap().into(),
            payload: value
                .get("payload")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|element| Payload::from_json(element))
                .collect(),
        }
    }

    pub fn emit_struct(&self) -> TokenStream {
        let comment = &self.description;
        let struct_name = quote::format_ident!("{}Struct", self.name.to_case(Case::Pascal));
        let variables: Vec<TokenStream> = self
            .payload
            .iter()
            .map(|variable| {
                let comment = variable
                    .description
                    .clone()
                    .unwrap_or("Not documented".to_string());
                let comment = comment.trim();
                let name = ident!(&variable.name);
                let typ = ident!(variable.typ.to_rust());
                quote! {
                    #[doc = #comment]
                    pub #name: #typ,
                }
            })
            .collect();
        quote! {
            #[derive(Debug, Clone, PartialEq, Default)]
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            #[doc = #comment]
            pub struct #struct_name {
                #[doc = "Information about message destiny and source"]
                pub header: PingProtocolHead,
                #(#variables)*
            }

        }
    }
}

pub fn emit_protocol_wrapper() -> TokenStream {
    quote! {
        #[derive(Debug, Clone, PartialEq, Default)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub struct PingProtocolHead {
            source_device_id: u8,
            destiny_device_id: u8,
        }
    }
}

/// Generate rust representation of ping-protocol message set with appropriate conversion methods
pub fn generate<R: Read, W: Write>(input: &mut R, output_rust: &mut W) {
    let messages = parse_description(input);
    let messages = messages
        .iter()
        .map(|(message_type, messages)| messages)
        .flatten()
        .collect::<HashMap<&String, &MessageDefinition>>();

    let message_enums = messages
        .iter()
        .map(|(name, _message)| {
            let pascal_message_name = ident!(name.to_case(Case::Pascal));
            let pascal_struct_name = quote::format_ident!("{}Struct", pascal_message_name);
            quote!(#pascal_message_name(#pascal_struct_name),)
        })
        .collect::<Vec<TokenStream>>();
    let message_enums = quote! {
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum Messages {
            #(#message_enums)*
        }
    };

    let message_tokens = messages
        .iter()
        .map(|(name, message)| message.emit_struct())
        .collect::<Vec<TokenStream>>();

    let protocol_wrapper = emit_protocol_wrapper();

    let code = quote! {
        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};

        #protocol_wrapper

        #message_enums

        #(#message_tokens)*
    };

    // rust file
    //let rust_tokens = description.emit_rust();
    writeln!(output_rust, "{}", code).unwrap();
}

fn parse_description(
    file: &mut dyn Read,
) -> HashMap<std::string::String, HashMap<std::string::String, MessageDefinition>> {
    let mut file_content = String::new();
    file.read_to_string(&mut file_content);
    let json: HashMap<String, HashMap<String, HashMap<String, serde_json::Value>>> =
        match serde_json::from_str(&file_content) {
            Ok(content) => content,
            Err(error) => panic!("{:#?}", error),
        };

    let message_categories = &json["messages"];
    let message_categories: HashMap<String, HashMap<String, MessageDefinition>> =
        message_categories
            .iter()
            .map(|(category, message)| {
                (
                    category.clone(),
                    message
                        .iter()
                        .map(|(message_name, value)| {
                            (
                                message_name.clone(),
                                MessageDefinition::from_json(message_name, value),
                            )
                        })
                        .collect::<HashMap<String, MessageDefinition>>(),
                )
            })
            .collect();

    return message_categories;
}
