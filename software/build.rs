// ABOUTME: Build script for compiling Protocol Buffer definitions using tonic-build
// ABOUTME: Generates Rust code from proto files for gRPC communication support

use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Configure check-cfg for has_grpc cfg directive
    println!("cargo::rustc-check-cfg=cfg(has_grpc)");

    // Only compile protobuf when the grpc features are enabled
    #[cfg(any(feature = "grpc-client", feature = "grpc-server"))]
    {
        // PROTOC binary should be available in PATH or set via PROTOC env var
        // tonic-prost-build will handle protoc discovery automatically

        // Only compile if proto file exists
        if std::path::Path::new("proto/qollective.proto").exists() {
            println!("cargo:rerun-if-changed=proto/qollective.proto");

            // Create the output directory if it doesn't exist
            let out_dir = PathBuf::from("src/generated");
            std::fs::create_dir_all(&out_dir).ok();

            // Use tonic-prost-build for tonic 0.14 compatibility
            tonic_prost_build::compile_protos("proto/qollective.proto")?;

            println!("cargo:rustc-cfg=has_grpc");
        }
    }

    Ok(())
}
