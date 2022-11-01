extern crate napi_build;
extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .out_dir("src/generated")
        .compile_protos(
            &["schema/osmformat.proto", "schema/fileformat.proto"],
            &["schema"],
        )
        .expect("failed to compile protobuf");

    napi_build::setup();
}
