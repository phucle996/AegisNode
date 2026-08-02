// build.rs cho crate aegis-rpc
// Tự động compile .proto files thành Rust code tại build time bằng tonic-build

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile agent.proto → gen AgentServiceServer / AgentServiceClient
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["proto/agent.proto"],
            &["proto"],
        )?;

    // Compile controller.proto → gen ControllerServiceServer / ControllerServiceClient
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["proto/controller.proto"],
            &["proto"],
        )?;

    Ok(())
}
