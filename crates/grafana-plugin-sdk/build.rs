fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::compile_protos("proto/pluginv2.proto")?;
  Ok(())
}
