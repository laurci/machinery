use machinery_build::pipeline;

fn main() {
    // println!("do nothing");
    pipeline::default("./src")
        .add_files("./**/*.rs")
        .export_to_dir("./bindings/")
        .enable_debug_comments()
        .with_custom_types(vec!["Custom = string"])
        .with_custom_header("/* custom header */")
        .with_custom_footer("/* custom_footer */")
        // .with_base_crate_path("crate::api")
        .build()
        .unwrap();
}
