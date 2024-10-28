fn main() {
    tonic_build::configure()
        .compile_protos(&["proto/helloworld.proto"], &["/proto"])
        .unwrap();
}
