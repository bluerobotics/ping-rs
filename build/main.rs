#[macro_use]
extern crate quote;
extern crate serde_json;

mod binder;
mod parser;

use std::env;
use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn main() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Update and init submodule
    match Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .current_dir(&src_dir)
        .status()
    {
        Ok(_) => {}
        Err(error) => eprintln!("{}", error),
    }

    let mut definitions_dir = src_dir.to_path_buf();
    definitions_dir.push("build/ping-protocol/src/definitions");

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut modules = vec![];

    for entry in read_dir(&definitions_dir).expect("could not read definitions directory") {
        let entry = entry.expect("could not read directory entry");

        let definition_file = entry.file_name();
        let module_name: String = definition_file.to_str().unwrap().into();

        if module_name.contains("pinghf") {
            continue;
        }

        let mut definition_rs = PathBuf::from(&module_name);
        definition_rs.set_extension("rs");

        modules.push(
            PathBuf::from(&module_name)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );

        let in_path = Path::new(&definitions_dir).join(&definition_file);
        let mut in_file = File::open(&in_path).unwrap();

        let dest_path = Path::new(&out_dir).join(definition_rs);
        let mut out_file = File::create(&dest_path).unwrap();

        // generate code
        parser::generate(&mut in_file, &mut out_file);

        // format code
        match Command::new("rustfmt")
            .arg(dest_path.as_os_str())
            .current_dir(&out_dir)
            .status()
        {
            Ok(_) => (),
            Err(error) => eprintln!("{}", error),
        }

        // Re-run build if definition file changes
        println!("cargo:rerun-if-changed={}", entry.path().to_string_lossy());
    }

    // output mod.rs
    {
        let dest_path = Path::new(&out_dir).join("mod.rs");
        let mut outf = File::create(&dest_path).unwrap();

        // generate code
        binder::generate(modules, &mut outf);

        // format code
        match Command::new("rustfmt")
            .arg(dest_path.as_os_str())
            .current_dir(&out_dir)
            .status()
        {
            Ok(_) => (),
            Err(error) => eprintln!("{}", error),
        }
    }
}
