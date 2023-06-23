use machinery_build::pipeline;

fn main() {
    // println!("do nothing");
    let mut build = pipeline::default("./src")
        .add_files("./**/*.rs")
        .export_to_dir("./bindings/")
        .with_custom_types(vec!["Custom = string"])
        .with_custom_header("/* custom header */")
        .with_custom_footer("/* custom_footer */")
        .to_owned();

    if cfg!(feature = "debug") {
        build.enable_debug_comments().enable_introspection();
    }

    // .with_base_crate_path("crate::api")
    build.build().unwrap();
}
