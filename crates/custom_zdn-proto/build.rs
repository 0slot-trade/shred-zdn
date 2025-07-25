use tonic_build::configure;

fn main() {
    configure().compile(&[
        "protos/relay.proto",
        // "protos/types.proto",
    ], &["protos"]).unwrap();
}