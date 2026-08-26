fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::build();

    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", &protoc_path);

    let proto_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto");

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[
                proto_dir.join("common.proto"),
                proto_dir.join("marketdata.proto"),
                proto_dir.join("instruments.proto"),
            ],
            &[proto_dir],
        )?;

    println!("cargo:rerun-if-changed=proto/common.proto");
    println!("cargo:rerun-if-changed=proto/marketdata.proto");
    println!("cargo:rerun-if-changed=proto/instruments.proto");
    println!("cargo:rerun-if-changed=proto/google/api/field_behavior.proto");

    Ok(())
}
