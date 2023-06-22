use machinery_build::pipeline;

fn main() {
    // println!("do nothing");
    pipeline::default("./src")
        .add_files("./**/*.rs")
        .export_to_dir("./bindings/")
        // .with_base_crate_path("crate::api")
        .build()
        .unwrap();
}
