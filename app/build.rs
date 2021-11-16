use std::path::Path;

fn main() {
    let protobuf_out = Path::new("../libsignal-service-rs/libsignal-service/src/protobuf").to_owned();
    let protobuf_include = Path::new("../libsignal-service-rs/libsignal-service/src/protobuf/proto").to_owned();

    // Build script does not automagically rerun when a new protobuf file is added.
    // Directories are checked against mtime, which is platform specific
    println!("cargo:rerun-if-changed=protobuf_include");
    // Adding src/proto.rs means an extra `include!` will trigger a rerun. This is on best-effort
    // basis.
    println!("cargo:rerun-if-changed=src/proto.rs");

    let input: Vec<_> = protobuf_include
        .read_dir()
        .expect("protobuf directory")
        .filter_map(|entry| {
            let entry = entry.expect("readable protobuf directory");
            let path = entry.path();
            if Some("proto")
                == path.extension().and_then(std::ffi::OsStr::to_str)
            {
                assert!(path.is_file());
                println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
                Some(path)
            } else {
                None
            }
        })
        .collect();
    let mut config = prost_build::Config::default();
    config.out_dir(&protobuf_out);

    config.compile_protos(&input, &[protobuf_include]).unwrap();
}
