use proc_macro2::Literal;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn parse(errcodes: &str) -> impl DoubleEndedIterator<Item = (u32, &str)> {
    let from_start = errcodes
        .splitn(2, "switch_known_errcodes = {")
        .nth(1)
        .unwrap();
    let region = from_start.split("}\n").next().unwrap().trim();
    region
        .lines()
        .filter(|line| line.len() > 0 && !line.trim().starts_with("#"))
        .map(|line| {
            let mut fields = line.splitn(2, ":");
            let code = u32::from_str_radix(&fields.next().unwrap().trim()[2..], 16).unwrap();
            let message = fields
                .next()
                .unwrap()
                .trim_matches(&[' ', '"', ',', '\''] as &[char]);
            (code, message)
        })
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let mut map = phf_codegen::Map::new();

    let deduped: HashMap<_, _> = parse(include_str!("assets/errcodes.py")).rev().collect();
    for (k, v) in deduped {
        map.entry(k, &Literal::string(v).to_string());
    }

    writeln!(
        &mut file,
        "static ERROR_CODES: phf::Map<u32, &'static str> = \n{};\n",
        map.build(),
    )
    .unwrap();

    drop(file);

    let include_path = "/opt/devkitpro/libnx/include/";
    cpp_build::Config::new()
        .include(include_path)
        .build("src/main.rs");
}
