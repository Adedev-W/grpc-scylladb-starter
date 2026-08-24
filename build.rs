fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    unsafe { std::env::set_var("PROTOC", protoc) };

    println!("cargo:rerun-if-changed=proto/channel.proto");
    println!("cargo:rerun-if-changed=proto/auth.proto");
    tonic_prost_build::configure()
        .compile_protos(&["proto/channel.proto", "proto/auth.proto"], &["proto"])?;

    Ok(())
}
