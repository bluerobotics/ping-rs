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
struct VectorType {
    size_type: Option<PayloadType>,
    data_type: PayloadType,
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
    VECTOR(Box<VectorType>),
}

impl PayloadType {
    pub fn from_string(name: &str) -> Self {
        match name {
            "char" => PayloadType::CHAR,
            "u8" | "uint8_t" => PayloadType::U8,
            "u16" | "uint16_t" => PayloadType::U16,
            "u32" | "uint32_t" => PayloadType::U32,
            "i8" | "int8_t" => PayloadType::I8,
            "i16" | "int16_t" => PayloadType::I16,
            "i32" | "int32_t" => PayloadType::I32,
            "float" => PayloadType::F32,
            "vector" => panic!("Can't convert vector with from_string."),
            something => panic!("No available type: {:#?}", something),
        }
    }

    pub fn from_json(value: &serde_json::Value) -> Self {
        let typ = value.get("type").unwrap().as_str().unwrap();
        if typ.contains("vector") {
            return PayloadType::VECTOR(Box::new(VectorType {
                data_type: PayloadType::from_string(
                    value.pointer("/vector/datatype").unwrap().as_str().unwrap(),
                ),
                size_type: match value.pointer("/vector/sizetype") {
                    Some(value) => Some(PayloadType::from_string(value.as_str().unwrap())),
                    None => None,
                },
            }));
        }

        return PayloadType::from_string(&typ);
    }

    pub fn to_rust(&self) -> TokenStream {
        match self {
            PayloadType::CHAR => quote! {char},
            PayloadType::U8 => quote! {u8},
            PayloadType::U16 => quote! {u16},
            PayloadType::U32 => quote! {u32},
            PayloadType::I8 => quote! {i8},
            PayloadType::I16 => quote! {i16},
            PayloadType::I32 => quote! {i32},
            PayloadType::F32 => quote! {f32},
            PayloadType::VECTOR(vector) => panic!("Can't convert vector to rust."),
        }
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
    pub fn emit_struct_variable(&self) -> TokenStream {
        let name = ident!(self.name);

        if let PayloadType::VECTOR(vector) = &self.typ {
            let data_type = vector.data_type.to_rust();
            if let Some(size_type) = &vector.size_type {
                let length_name = quote::format_ident!("{}_length", self.name);
                let length_type = size_type.to_rust();
                return quote! {
                    pub #length_name: #length_type,
                    pub #name: Vec<#data_type>,
                };
            }

            return quote! {
                #name: Vec<#data_type>,
            };
        }

        let typ = self.typ.to_rust();
        quote! {
            pub #name: #typ,
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
                //let name = ident!(&variable.name);
                let variable = variable.emit_struct_variable();
                quote! {
                    #[doc = #comment]
                    #variable
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
            pub source_device_id: u8,
            pub destiny_device_id: u8,
        }
    }
}

fn emit_ping_message(messages: HashMap<&String, &MessageDefinition>) -> TokenStream {
    let message_enums_name = messages
        .iter()
        .map(|(name, _message)| {
            let pascal_message_name = ident!(name.to_case(Case::Pascal));
            quote!(Messages::#pascal_message_name(..) => #name,)
        })
        .collect::<Vec<TokenStream>>();

    let message_enums_id = messages
        .iter()
        .map(|(name, message)| {
            let pascal_message_name = ident!(name.to_case(Case::Pascal));
            let id = message.id;
            let id = quote!(#id);
            quote!(Messages::#pascal_message_name(..) => #id,)
        })
        .collect::<Vec<TokenStream>>();

    let message_enums_name_id = messages
    .iter()
    .map(|(name, message)| {
        let id = message.id;
        let id = quote!(#id);
        quote!(#name  => Ok(#id),)
    })
    .collect::<Vec<TokenStream>>();

    quote! {
        impl PingMessage for Messages {
            fn message_name(&self) -> &'static str {
                match self {
                    #(#message_enums_name)*
                }
            }

            fn message_id(&self) -> u16 {
                match self {
                    #(#message_enums_id)*
                }
            }

            fn message_id_from_name(name: &str) -> Result<u16, &'static str> {
                match name {
                    #(#message_enums_name_id)*
                    _ => Err("Invalid message name."),
                }
            }
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

    let ping_message = emit_ping_message(messages);

    let code = quote! {
        use crate::serialize::PingMessage;

        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};

        #protocol_wrapper

        #message_enums

        #(#message_tokens)*

        #ping_message
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
