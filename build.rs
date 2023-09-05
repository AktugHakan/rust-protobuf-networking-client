use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["request.proto", "response.proto"],
        &["/home/ahmet/Documents/RustProtobufNetworking/protobuf"],
    )?;
    Ok(())
}
