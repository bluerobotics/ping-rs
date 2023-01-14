use std::io::Write;

use quote;

pub fn generate<W: Write>(modules: Vec<String>, out: &mut W) {
    dbg!(&modules);
    let modules_tokens = modules.into_iter().map(|module| {
        let file_name = module.clone() + ".rs";
        let module_ident = quote::format_ident!("{}", module.clone());

        quote! {
            pub mod #module_ident {
                include!(#file_name);
            }
        }
    });

    let tokens = quote! {
        #(#modules_tokens)*
    };

    writeln!(out, "{}", tokens).unwrap();
}
