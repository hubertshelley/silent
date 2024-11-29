fn main() {
    tonic_build::configure()
        .compile_protos(&["proto/hello_world.proto"], &["/proto"])
        .unwrap();
}
