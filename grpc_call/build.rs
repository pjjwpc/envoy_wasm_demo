fn main() {
    let proto_files = vec!["./protos/hello.proto"];
    protoc_rust::Codegen::new()
        .out_dir("./src/pb")
        .inputs(proto_files)
        .run()
        .expect("build error");
}
