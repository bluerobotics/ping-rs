use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use std::io::Write;

use quote;

pub fn generate<W: Write>(modules: Vec<String>, out: &mut W) {
    dbg!(&modules);
    let modules_tokens = modules
        .clone()
        .into_iter()
        .map(|module| {
            let file_name = module.clone() + ".rs";
            let module_ident = quote::format_ident!("{module}");

            quote! {
                pub mod #module_ident {
                    include!(#file_name);
                }
            }
        })
        .collect::<Vec<TokenStream>>();

    let enum_tokens = modules
        .clone()
        .into_iter()
        .map(|module| {
            let pascal_case_ident = quote::format_ident!("{}", module.to_case(Case::Pascal));
            let module_ident = quote::format_ident!("{module}");

            quote! {
                #pascal_case_ident(#module_ident::Messages),
            }
        })
        .collect::<Vec<TokenStream>>();

    let deserialize_tokens_vec = modules.clone().into_iter().map(|module| {
        let pascal_case_ident = quote::format_ident!("{}", module.to_case(Case::Pascal));
        let module_ident = quote::format_ident!("{module}");
        quote! {
            if let Ok(message) =
                <#module_ident::Messages as DeserializeGenericMessage>::deserialize(protocol_message.message_id, &protocol_message.payload)
            {
                return Ok(Messages::#pascal_case_ident(message));
            }
        }
    }).collect::<Vec<TokenStream>>();

    let deserialize_tokens = modules.clone().into_iter().map(|module| {
        let pascal_case_ident = quote::format_ident!("{}", module.to_case(Case::Pascal));
        let module_ident = quote::format_ident!("{module}");
        quote! {
            if let Ok(message) = <#module_ident::Messages as DeserializeGenericMessage>::deserialize(message.message_id, &message.payload) {
                return Ok(Messages::#pascal_case_ident(message));
            }
        }
    }).collect::<Vec<TokenStream>>();

    let enum_tokens_inner = modules
        .clone()
        .into_iter()
        .map(|module| {
            let pascal_case_ident = quote::format_ident!("{}", module.to_case(Case::Pascal));

            quote! {
                Self::#pascal_case_ident(inner_enum) => inner_enum.inner(),
            }
        })
        .collect::<Vec<TokenStream>>();

    let enum_ident = quote! {
        pub enum Messages {
            #(#enum_tokens)*
        }

        impl Messages {
            pub fn inner<T: 'static>(&self) -> Option<&T> {
                match self {
                    #(#enum_tokens_inner)*
                }
            }
        }
    };

    let try_from_ident = quote! {
        impl TryFrom<&ProtocolMessage> for Messages {
            type Error = String; // TODO: define error types for each kind of failure

            fn try_from(message: &ProtocolMessage) -> Result<Self, Self::Error> {
                if !message.has_valid_crc() {
                    return Err(format!(
                        "Missmatch crc, expected: 0x{:04x}, received: 0x{:04x}",
                        message.calculate_crc(),
                        message.checksum
                    ));
                }

                #(#deserialize_tokens)*

                Err("Unknown message".into())
            }
        }

        impl TryFrom<&Vec<u8>> for Messages {
            type Error = String; // TODO: define error types for each kind of failure

            fn try_from(buffer: &Vec<u8>) -> Result<Self, Self::Error> {
                const MIN_MSG_SIZE: usize = 10;

                // Parse start1 and start2
                if !((buffer[0] == HEADER[0]) && (buffer[1] == HEADER[1])) {
                    return Err(format!("Message should start with \"BR\" ASCII sequence, received: [{0}({:0x}), {1}({:0x})]", buffer[0], buffer[1]));
                }

                if buffer.len() < MIN_MSG_SIZE {
                    return Err(format!("Message is too short, should be at least {MIN_MSG_SIZE} bytes").into());
                }

                let payload_length = u16::from_le_bytes([buffer[2], buffer[3]]);
                if payload_length as usize + MIN_MSG_SIZE != buffer.len() {
                    return Err(format!(
                        "Payload length does not match, expected: {payload_length}, received: {}",
                        buffer.len() - MIN_MSG_SIZE
                    ));
                }

                let protocol_message = ProtocolMessage {
                    payload_length,
                    message_id: u16::from_le_bytes([buffer[4], buffer[5]]),
                    src_device_id: buffer[6],
                    dst_device_id: buffer[7],
                    payload: buffer[8..(8 + payload_length) as usize].into(),
                    checksum: u16::from_le_bytes([
                        buffer[(8 + payload_length) as usize],
                        buffer[(8 + payload_length + 1) as usize],
                    ]),
                };

                if !protocol_message.has_valid_crc() {
                    return Err(format!(
                        "Missmatch crc, expected: 0x{:04x}, received: 0x{:04x}",
                        protocol_message.calculate_crc(),
                        protocol_message.checksum
                    ));
                }

                #(#deserialize_tokens_vec)*

                Err("Unknown message".into())
            }
        }
    };

    let tokens = quote! {
        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};

        #(#modules_tokens)*

        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
        #enum_ident

        #try_from_ident
    };

    writeln!(out, "{tokens}").unwrap();
}
