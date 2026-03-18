fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let proto = "proto/prior/gate.proto";

    println!("cargo:rerun-if-changed={proto}");
    println!("cargo:rerun-if-env-changed=PROTOC");

    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc);
    config.compile_protos(&[proto], &["proto"])?;
    Ok(())
}

