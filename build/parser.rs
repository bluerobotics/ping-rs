use std::collections::HashMap;
use std::io::{Read, Write};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
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
            PayloadType::VECTOR(_vector) => panic!("Can't convert vector to rust."),
        }
    }

    pub fn to_size(&self) -> usize {
        match self {
            PayloadType::CHAR | PayloadType::U8 | PayloadType::I8 => 1,
            PayloadType::U16 | PayloadType::I16 => 2,
            PayloadType::U32 | PayloadType::I32 | PayloadType::F32 => 4,
            PayloadType::VECTOR(_) => 0,
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

            // There is no size_type, so it should be a string
            return quote! {
                #name: String,
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

        // Serialization part
        let mut sum_quote = None;
        let mut variables_serialized: Vec<TokenStream> = vec![];
        let mut sum = 0;
        for pay in &self.payload {
            let size = pay.typ.to_size();
            let name = quote::format_ident!("{}", pay.name);

            if let PayloadType::VECTOR(vector) = &pay.typ {
                if let Some(size_type) = &vector.size_type {
                    let length_name = quote::format_ident!("{}_length", name);
                    let content_size = size_type.to_size();

                    let final_size = sum + content_size;
                    variables_serialized.push(quote! {
                        buffer[#sum..#final_size].copy_from_slice(&self.#length_name.to_le_bytes());
                    });
                    sum = final_size;

                    variables_serialized.push(quote! {
                        for (index, value) in self.#name.iter().enumerate() {
                            buffer[(#sum + index * #content_size)..(#sum + (1 + index) * #content_size)].copy_from_slice(&value.to_le_bytes());
                        }
                    });

                    sum_quote = Some(quote! {
                        #sum + (self.#length_name as usize * self.#name.len()) as usize
                    })
                } else {
                    // We are probably dealing with a string since size_type is empty
                    variables_serialized.push(quote! {
                        let string_size = self.#name.len();
                        let final_size = #sum + string_size;
                        buffer[#sum..final_size].copy_from_slice(self.#name.as_bytes());
                        buffer[final_size] = 0;
                    });

                    sum_quote = Some(quote! {
                        final_size + 1usize
                    })
                }

                // Vector should be the last element, we should not care about sum
                continue;
            }

            let final_size = sum + size;
            variables_serialized.push(quote! {
                buffer[#sum..#final_size].copy_from_slice(&self.#name.to_le_bytes());
            });
            sum = final_size;
        }

        let sum_quote = sum_quote.or_else(|| Some(quote! { #sum }));

        // Descerialization part
        let mut b: usize = 0; // current byte
        let variables_deserialized: Vec<TokenStream> = self
            .payload
            .iter()
            .map(|field| {
                let name = ident!(field.name);
                match &field.typ {
                    PayloadType::I8 | PayloadType::U8 | PayloadType::CHAR => {
                        let value = quote! {
                            #name: payload[#b].into(),
                        };
                        b += field.typ.to_size();
                        value
                    }
                    PayloadType::U16 | PayloadType::I16 | PayloadType::U32 | PayloadType::I32 | PayloadType::F32 => {
                        let data_type = field.typ.to_rust();
                        let data_size = field.typ.to_size();
                        let field_token = quote! {
                            #name: #data_type::from_le_bytes(payload[#b..#b + #data_size].try_into().expect("Wrong slice length")),
                        };
                        b += data_size;
                        field_token
                    }
                    PayloadType::VECTOR(vector) => {
                        let data_type = vector.data_type.to_rust();
                        let data_size = vector.data_type.to_size();
                        if let Some(size_type) = &vector.size_type {
                            let length_name = quote::format_ident!("{}_length", field.name);
                            let length_type = size_type.to_rust();
                            let length = self.payload.len();
                            let field_token = {
                                let value = match vector.data_type {
                                    PayloadType::CHAR |
                                    PayloadType::U8 |
                                    PayloadType::I8 => quote! {
                                        payload[#b..#b + payload.len()].to_vec()
                                    },
                                    PayloadType::U16 |
                                    PayloadType::U32 |
                                    PayloadType::I16 |
                                    PayloadType::I32 |
                                    PayloadType::F32 => quote! {
                                        payload[#b..#b + payload.len()]
                                            .chunks_exact(#data_size)
                                            .into_iter()
                                            .map(|a| u16::from_le_bytes((*a).try_into().expect("Wrong slice length")))
                                            .collect::<Vec<#data_type>>()
                                    },
                                    PayloadType::VECTOR(_) => unimplemented!("Vector of vectors are not supported"),
                                };

                                quote! {
                                    #length_name: payload.len() as #length_type,
                                    #name: #value,
                                }
                            };
                            b += length;
                            field_token
                        } else {
                            let length = self.payload.len();
                            let field_token = quote! {
                                #name: String::from_utf8(payload[#b..#b + payload.len()].to_vec()).unwrap(),
                            };
                            b += length;
                            field_token
                        }
                    }
                }
            })
            .collect();

        quote! {
            #[derive(Debug, Clone, PartialEq, Default)]
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            #[doc = #comment]
            pub struct #struct_name {
                #(#variables)*
            }

            impl Serialize for #struct_name {
                fn serialize(&self, buffer: &mut [u8]) -> usize {
                    #(#variables_serialized)*
                    #sum_quote
                }
            }

            impl Deserialize for #struct_name {
                fn deserialize(payload: &[u8]) -> Result<Self, &'static str> {
                    Ok(Self {
                        #(#variables_deserialized)*
                    })
                }
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

    let message_enums_serialize = messages
        .iter()
        .map(|(name, _message)| {
            let pascal_message_name = ident!(name.to_case(Case::Pascal));
            quote!(Messages::#pascal_message_name(content) => content.serialize(buffer),)
        })
        .collect::<Vec<TokenStream>>();

    let message_enums_deserialize = messages
        .iter()
        .map(|(name, message)| {
            let pascal_message_name = ident!(name.to_case(Case::Pascal));
            let struct_name = quote::format_ident!("{}Struct", pascal_message_name);
            let id = message.id;

            quote! {
                #id => Messages::#pascal_message_name(#struct_name::deserialize(payload)?),
            }
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

        impl Serialize for Messages {
            fn serialize(&self, buffer: &mut [u8]) -> usize {
                match self {
                    #(#message_enums_serialize)*
                }
            }
        }

        impl Deserialize for Messages {
            fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
                // Parse start1 and start2
                if !((buffer[0] == b'B') && (buffer[1] == b'R')) {
                    return Err("Message should start with \"BR\" ASCII sequence");
                }

                // Get the package data
                let payload_length = u16::from_le_bytes([buffer[2], buffer[3]]);
                let message_id = u16::from_le_bytes([buffer[4], buffer[5]]);
                let _src_device_id = buffer[6];
                let _dst_device_id = buffer[7];
                let payload = &buffer[8..(8 + payload_length) as usize];
                let _checksum = u16::from_le_bytes([
                    buffer[(payload_length + 1) as usize],
                    buffer[(payload_length + 2) as usize],
                ]);

                // Parse the payload
                Ok(match message_id {
                    #(#message_enums_deserialize)*
                    _ => {
                        return Err(&"Unknown message id");
                    }
                })
            }
        }


    }
}

/// Generate rust representation of ping-protocol message set with appropriate conversion methods
pub fn generate<R: Read, W: Write>(input: &mut R, output_rust: &mut W) {
    let messages = parse_description(input);
    let messages = messages
        .iter()
        .map(|(_message_type, messages)| messages)
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
        .map(|(_name, message)| message.emit_struct())
        .collect::<Vec<TokenStream>>();

    let protocol_wrapper = emit_protocol_wrapper();

    let ping_message = emit_ping_message(messages);

    let code = quote! {
        use crate::serialize::PingMessage;
        use crate::serialize::Serialize;
        use crate::serialize::Deserialize;
        use std::convert::TryInto;

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
    let _ = file.read_to_string(&mut file_content);
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
