use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use protox::prost_reflect::DescriptorPool;
use walkdir::WalkDir;

fn contract_files(proto_dir: &Path) -> Vec<String> {
    let mut files: Vec<String> = WalkDir::new(proto_dir.join("yunheshiguang"))
        .into_iter()
        .map(|entry| entry.expect("proto directory is unreadable"))
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "proto")
        })
        .map(|entry| {
            entry
                .path()
                .strip_prefix(proto_dir)
                .expect("proto file sits outside the proto directory")
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect();
    files.sort();
    files
}

fn local_api_procedures(pool: &DescriptorPool) -> String {
    let option = pool
        .get_extension_by_name("yunheshiguang.rpc.v1.local_api")
        .expect("the local_api method option is defined");
    let mut procedures: Vec<String> = pool
        .services()
        .flat_map(|service| service.methods().collect::<Vec<_>>())
        .filter(|method| {
            let options = method.options();
            options.has_extension(&option) && options.get_extension(&option).as_bool() == Some(true)
        })
        .map(|method| format!("/{}/{}", method.parent_service().full_name(), method.name()))
        .collect();
    procedures.sort();

    format!("pub const LOCAL_API_PROCEDURES: &[&str] = &{procedures:?};\n")
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let proto_dir = manifest_dir.join("../../proto");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is required"));

    println!("cargo:rerun-if-changed={}", proto_dir.display());

    let files = contract_files(&proto_dir);
    let mut compiler = protox::Compiler::new([&proto_dir]).expect("proto directory is unusable");
    compiler.include_imports(true);
    compiler
        .open_files(&files)
        .expect("proto compilation failed");
    let descriptor_set = out_dir.join("contract.binpb");
    fs::write(&descriptor_set, compiler.encode_file_descriptor_set())
        .expect("descriptor set could not be written");
    fs::write(
        out_dir.join("local_api.rs"),
        local_api_procedures(&compiler.descriptor_pool()),
    )
    .expect("the local API procedures could not be written");

    connectrpc_build::Config::new()
        .descriptor_set(&descriptor_set)
        .files(&files)
        .include_file("_connectrpc.rs")
        .emit_rerun_directives(false)
        .compile()
        .expect("contract generation failed");
}
