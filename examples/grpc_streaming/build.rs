fn main() {
    tonic_build::configure()
        .compile_protos(&["proto/echo.proto"], &["/proto"])
        .unwrap();
}
