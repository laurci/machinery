use glob_match::glob_match;
use std::path;
use walkdir::WalkDir;

use crate::analyzer::{analyze_file, AnalyzeResult};

const HEADER: &str = "
export interface Transport {
	send(fn: string, args: string): Promise<string>;
}

function handleResult(result: string) {
	const json = JSON.parse(result);
	if (json.error) {
		throw new Error(json.error);
	}
	return json.result;
}

type Option<T> = T | undefined;
type Result<T> = T;
type Vec<T> = T[];

type String = string;
type Void = void;

type u8 = number;
type u16 = number;
type u32 = number;
type u64 = number;
type usize = number;

type i8 = number;
type i16 = number;
type i32 = number;
type i64 = number;
type isize = number;

type f32 = number;
type f64 = number;

type bool = boolean;
";

#[derive(Debug)]
pub enum Error {
    MissingExportDir,
    FailedToParseFile(String),
    FailedToWriteFile(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::MissingExportDir => write!(f, "Missing export dir"),
            Error::FailedToParseFile(file) => write!(f, "Failed to parse file: {}", file),
            Error::FailedToWriteFile(file) => write!(f, "Failed to write file: {}", file),
        }
    }
}

pub struct Pipeline {
    root_dir: String,
    base_crate_path: String,
    files: Vec<String>,
    export_dir: Option<String>,
}

impl Pipeline {
    pub fn add_files(&mut self, glob: &str) -> &mut Self {
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.root_dir) {
            let path = entry.unwrap().path().to_owned();
            if path.is_file() {
                let path = path.to_str().unwrap().to_owned();
                if glob_match(glob, &path) {
                    files.push(path);
                }
            }
        }
        self.files = files;

        self
    }

    pub fn with_base_crate_path(&mut self, path: &str) -> &mut Self {
        self.base_crate_path = path.to_owned();

        self
    }

    pub fn export_to_dir(&mut self, path: &str) -> &mut Self {
        self.export_dir = Some(path.to_owned());

        self
    }

    pub fn build_ts_client(&mut self, result: &AnalyzeResult) -> Result<(), Error> {
        let export_dir = self.export_dir.clone().ok_or(Error::MissingExportDir)?;
        let export_dir = path::Path::new(&export_dir);

        let mut code = String::new();

        code.push_str(&format!("/*\n{:#?}\n*/\n", &result));
        code.push_str(HEADER);

        for message in &result.messages {
            code.push_str(&message.code);
            code.push('\n');
        }

        code.push_str(
            "export function createClient(transport: Transport) {
\tlet obj0 = {};
",
        );

        let mut obj_count: u32 = 1;

        // create a tree from the services location eg crate::api::greeting::hello -> { crate: { api: { greeting: { hello: {} } } } }
        for service in &result.services {
            let base_path = service
                .location
                .replace(&format!("{}::", &self.base_crate_path), "");

            let obj_path = base_path.split("::").collect::<Vec<_>>().join(": { ");
            let obj_path = format!("{{ {}: {{", obj_path);
            let obj_end = "}".repeat(base_path.split("::").count() + 1);

            let service_args = service
                .arguments
                .clone()
                .into_iter()
                .map(|arg| {
                    let arg = arg.split(":").collect::<Vec<_>>();
                    (arg[0].to_owned(), arg[1].to_owned())
                })
                .collect::<Vec<(String, String)>>();

            let call_args = service_args
                .into_iter()
                .map(|(name, _)| format!("{name} ?? null"))
                .collect::<Vec<_>>()
                .join(", ");

            code.push_str(&format!(
                "\tlet obj{} = Object.assign(obj{}, {} async {}({}): Promise<{}> {{ return handleResult(await transport.send(\"{}\", JSON.stringify([{}]))); }} {});\n",
                obj_count,
                obj_count - 1,
                obj_path,
                service.name,
                service.arguments.join(", "),
                service.return_type,
                format!("{}::{}", service.location.replace("crate::", ""), service.name),
                call_args,
                obj_end
            ));

            obj_count += 1;
        }

        code.push_str(&format!("\treturn obj{};\n}};\n", obj_count - 1));

        std::fs::create_dir_all(export_dir).unwrap();

        let file_path = export_dir.join("index.ts");
        std::fs::write(file_path, code)
            .map_err(|_| Error::FailedToWriteFile("index.ts".to_owned()))?;

        Ok(())
    }

    pub fn build_rust_handler(&mut self, result: &AnalyzeResult) -> Result<(), Error> {
        let out_dir = std::env::var_os("OUT_DIR").unwrap();
        let dest_path = std::path::Path::new(&out_dir).join("machinery.rs");

        let mut code = String::new();

        let mut handlers: Vec<String> = vec![];

        code.push_str("mod __machinery {\n");

        for service in &result.services {
            let full_name = &format!("{}::{}", service.location, service.name);
            // let name = &service.name;
            let base_name = &full_name.replace(&format!("{}::", self.base_crate_path), "");
            let ident_name = &base_name.replace("::", "_");
            let call_name = full_name.replace("crate::", "");

            handlers.push(format!(
                "\"{call_name}\" => handle_{ident_name}(json_input).await,"
            ));

            let args = service.arguments.clone().into_iter().map(|arg| {
                let arg = arg.split(":").collect::<Vec<_>>();
                (arg[0].to_owned(), arg[1].to_owned())
            });

            if args.len() == 0 {
                code.push_str(&format!(
                    "
    async fn handle_{ident_name}(_json_input: String) -> String {{
        let res_output = {full_name}().await;
        if res_output.is_err()  {{
            return format!(\"{{{{ \\\"error\\\": \\\"{{}}\\\" }}}}\", res_output.unwrap_err());
        }}
        
        let output = res_output.unwrap();
        let output = machinery::json::to_string(&output);

        if output.is_ok() {{
            return format!(\"{{{{ \\\"result\\\": {{}} }}}}\", output.unwrap());
        }}

        return \"{{{{ \\\"error\\\": \\\"Failed to serialize output\\\" }}}}\".to_owned();
    }}
"
                ));
            } else {
                let arg_names = args
                    .clone()
                    .map(|(name, _)| format!("arg_{name}"))
                    .collect::<Vec<_>>();
                let arg_names_str = arg_names.join(", ");

                let arg_types = args.map(|(_, ty)| ty).collect::<Vec<_>>();
                let arg_types_str = arg_types.join(", ");

                code.push_str(&format!(
                    "
    async fn handle_{ident_name}(json_input: String) -> String {{
        let rest_input = machinery::json::from_str::<({arg_types_str},)>(&json_input);
        if rest_input.is_err() {{
            return \"{{{{ \\\"error\\\": \\\"Failed to deserialize input\\\" }}}}\".to_owned();
        }}
        let ({arg_names_str},) = rest_input.unwrap();

        let res_output = {full_name}({arg_names_str}).await;
        if res_output.is_err() {{
            return format!(\"{{{{ \\\"error\\\": \\\"{{}}\\\" }}}}\", res_output.unwrap_err());
        }}

        let output = res_output.unwrap();
        let output = machinery::json::to_string(&output);

        if output.is_ok() {{
            return format!(\"{{{{ \\\"result\\\": {{}} }}}}\", output.unwrap());
        }}

        return \"{{{{ \\\"error\\\": \\\"Failed to serialize output\\\" }}}}\".to_owned();
    }}
"
                ));
            }
        }

        code.push_str(
            format!(
                "
    pub async fn handle(fn_name: String, json_input: String) -> String {{
        match fn_name.as_str() {{
{}
            _ => return \"{{ \\\"error\\\": \\\"Unknown function\\\" }}\".to_owned(),
        }}
    }}
}}
",
                handlers.join("\n")
            )
            .as_str(),
        );

        std::fs::write(&dest_path, code)
            .map_err(|_| Error::FailedToWriteFile("machinery.rs".to_owned()))?;

        // panic!("{code}");

        //         std::fs::write(
        //             &dest_path,
        //             "
        // mod __machinery {
        //     async fn handle_hello(json_input: String) -> String {
        //         let res_input = serde_json::from_str::<(String,)>(&json_input);
        //         if res_input.is_err() {
        //             return \"{ \\\"error\\\": \\\"Failed to deserialize input\\\" }\".to_owned();
        //         }
        //         let (arg0,) = res_input.unwrap();
        //
        //         let res_output = crate::api::greeting::hello(arg0).await;
        //         if res_output.is_err() {
        //             return format!(\"{{ \\\"error\\\": \\\"{}\\\" }}\", res_output.unwrap_err());
        //         }
        //
        //         let output = res_output.unwrap();
        //         let output = serde_json::to_string(&output);
        //
        //         if output.is_ok() {
        //             return format!(\"{{ \\\"result\\\": {} }}\", output.unwrap());
        //         }
        //
        //         \"{ \\\"error\\\": \\\"Failed to serialize output\\\" }\".to_owned()
        //     }
        //
        //     pub async fn handle(fn_name: String, json_input: String) -> String {
        //         match fn_name.as_str() {
        //             \"crate::greeting::hello\" => handle_hello(json_input).await,
        //             _ => \"{ \\\"error\\\": \\\"Unknown function\\\" }\".to_owned(),
        //         }
        //     }
        // }
        //             ",
        //         )
        //         .unwrap();

        Ok(())
    }

    pub fn build(&mut self) -> Result<(), Error> {
        let mut combined_result = AnalyzeResult {
            file_location: "crate".to_owned(),
            services: Vec::new(),
            messages: Vec::new(),
        };

        for file_path in &self.files {
            let file_text = std::fs::read_to_string(file_path).unwrap();

            let ast = syn::parse_file(&file_text).unwrap();

            let result = analyze_file(&ast, &file_path);
            combined_result.services.extend(result.services);
            combined_result.messages.extend(result.messages);
        }

        self.build_ts_client(&combined_result)?;
        self.build_rust_handler(&combined_result)?;

        Ok(())
    }
}

pub fn default(root_dir: &str) -> Pipeline {
    Pipeline {
        root_dir: root_dir.to_owned(),
        files: Vec::new(),
        export_dir: None,
        base_crate_path: "crate".to_owned(),
    }
}
