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
                #module_ident::Messages::deserialize(protocol_message.message_id, &protocol_message.payload)
            {
                return Ok(Messages::#pascal_case_ident(message));
            }
        }
    }).collect::<Vec<TokenStream>>();

    let deserialize_tokens = modules.into_iter().map(|module| {
        let pascal_case_ident = quote::format_ident!("{}", module.to_case(Case::Pascal));
        let module_ident = quote::format_ident!("{module}");
        quote! {
            if let Ok(message) = #module_ident::Messages::deserialize(message.message_id, &message.payload) {
                return Ok(Messages::#pascal_case_ident(message));
            }
        }
    }).collect::<Vec<TokenStream>>();

    let enum_ident = quote! {
        pub enum Messages {
            #(#enum_tokens)*
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
                // Parse start1 and start2
                if !((buffer[0] == HEADER[0]) && (buffer[1] == HEADER[1])) {
                    return Err(format!("Message should start with \"BR\" ASCII sequence, received: [{0}({:0x}), {1}({:0x})]", buffer[0], buffer[1]));
                }

                let payload_length = u16::from_le_bytes([buffer[2], buffer[3]]);
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
        #(#modules_tokens)*

        #[derive(Debug)]
        #enum_ident

        #try_from_ident
    };

    writeln!(out, "{tokens}").unwrap();
}
