use glob_match::glob_match;
use std::path;
use walkdir::WalkDir;

use crate::analyzer::{analyze_file, AnalyzeResult, Service};

const INTROSPECTION_NAMESPACE: &str = "machinery_introspection";

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

export type Option<T> = T | undefined;
export type Result<T> = T;
export type Vec<T> = T[];

export type String = string;
export type Void = void;

export type u8 = number;
export type u16 = number;
export type u32 = number;
export type u64 = number;
export type usize = number;

export type i8 = number;
export type i16 = number;
export type i32 = number;
export type i64 = number;
export type isize = number;

export type f32 = number;
export type f64 = number;

export type bool = boolean;
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

#[derive(Clone)]
pub struct Pipeline {
    root_dir: String,
    base_crate_path: String,
    files: Vec<String>,
    export_dir: Option<String>,
    debug_comments: bool,
    introspection: bool,
    custom_types: Vec<String>,
    custom_header: Option<String>,
    custom_footer: Option<String>,
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

    pub fn enable_debug_comments(&mut self) -> &mut Self {
        self.debug_comments = true;

        self
    }

    pub fn enable_introspection(&mut self) -> &mut Self {
        self.introspection = true;

        self
    }

    pub fn with_base_crate_path(&mut self, path: &str) -> &mut Self {
        self.base_crate_path = path.to_owned();

        self
    }

    pub fn with_custom_types(&mut self, types: Vec<&str>) -> &mut Self {
        self.custom_types = types.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>();

        self
    }

    pub fn with_custom_header(&mut self, header: &str) -> &mut Self {
        self.custom_header = Some(header.to_owned());

        self
    }

    pub fn with_custom_footer(&mut self, footer: &str) -> &mut Self {
        self.custom_footer = Some(footer.to_owned());

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

        if self.debug_comments {
            code.push_str(&format!("/*\n{:#?}\n*/\n", &result));
        }

        if self.custom_header.is_some() {
            code.push_str(&self.custom_header.clone().unwrap());
        }
        code.push_str(HEADER);

        for custom_type in &self.custom_types {
            code.push_str(&format!("export type {};\n", custom_type));
        }

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
            if service.location.starts_with(INTROSPECTION_NAMESPACE) {
                continue;
            }

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

        if self.custom_footer.is_some() {
            code.push_str(&self.custom_footer.clone().unwrap());
        }

        std::fs::create_dir_all(export_dir).unwrap();

        let file_path = export_dir.join("index.ts");
        std::fs::write(file_path, code)
            .map_err(|_| Error::FailedToWriteFile("index.ts".to_owned()))?;

        Ok(())
    }

    pub fn build_rust_handler(&mut self, result: &mut AnalyzeResult) -> Result<(), Error> {
        let out_dir = std::env::var_os("OUT_DIR").unwrap();
        let dest_path = std::path::Path::new(&out_dir).join("machinery.rs");

        let mut code = String::new();

        let mut handlers: Vec<String> = vec![];

        code.push_str("mod __machinery {\n");

        if self.introspection {
            let ts_client_path = self.export_dir.clone().ok_or(Error::MissingExportDir)?;
            let ts_client_path = path::Path::new(&ts_client_path);
            let ts_client_path = ts_client_path.join("index.ts").to_str().unwrap().to_owned();

            code.push_str(&format!(
                "
    mod machinery_introspection {{
        pub async fn ts_client(_ctx: &machinery::context::Context) -> machinery::Result<String> {{
            let code = std::fs::read_to_string(\"{ts_client_path}\")?;
            Ok(code)
        }}
    }}
"
            ));
            result.services.push(Service {
                name: "ts_client".to_owned(),
                location: INTROSPECTION_NAMESPACE.to_owned(),
                arguments: vec![],
                return_type: "String".to_owned(),
            });
        }

        for service in &result.services {
            let full_name = &format!("{}::{}", service.location, service.name);
            // let name = &service.name;
            let base_name = &full_name.replace(&format!("{}::", self.base_crate_path), "");
            let ident_name = &base_name.replace("::", "_");
            let call_name = full_name.replace("crate::", "");

            handlers.push(format!(
                "\"{call_name}\" => handle_{ident_name}(&ctx, json_input).await,"
            ));

            let args = service.arguments.clone().into_iter().map(|arg| {
                let arg = arg.split(":").collect::<Vec<_>>();
                (arg[0].to_owned(), arg[1].to_owned())
            });

            if args.len() == 0 {
                code.push_str(&format!(
                    "
    async fn handle_{ident_name}(ctx: &machinery::context::Context, _json_input: String) -> String {{
        let res_output = {full_name}(ctx).await;
        if res_output.is_err()  {{
            return format!(\"{{{{ \\\"error\\\": \\\"{{}}\\\" }}}}\", res_output.unwrap_err());
        }}
        
        let output = res_output.unwrap();
        let output = machinery::json::to_string(&output);

        if output.is_ok() {{
            return format!(\"{{{{ \\\"result\\\": {{}} }}}}\", output.unwrap());
        }}

        return \"{{ \\\"error\\\": \\\"Failed to serialize output\\\" }}\".to_owned();
    }}
"
                ));
            } else {
                let arg_names = args
                    .clone()
                    .map(|(name, _)| format!("arg_{name}"))
                    .collect::<Vec<_>>();
                let arg_names_str = arg_names.join(", ");

                // let arg_types = args.map(|(_, ty)| ty).collect::<Vec<_>>();
                // let arg_types_str = arg_types.join(", ");

                code.push_str(&format!(
                    "
    async fn handle_{ident_name}(ctx: &machinery::context::Context, json_input: String) -> String {{
        let rest_input = machinery::json::from_str(&json_input);
        if rest_input.is_err() {{
            return \"{{ \\\"error\\\": \\\"Failed to deserialize input\\\" }}\".to_owned();
        }}
        let ({arg_names_str},) = rest_input.unwrap();

        let res_output = {full_name}(ctx, {arg_names_str}).await;
        if res_output.is_err() {{
            return format!(\"{{{{ \\\"error\\\": \\\"{{}}\\\" }}}}\", res_output.unwrap_err());
        }}

        let output = res_output.unwrap();
        let output = machinery::json::to_string(&output);

        if output.is_ok() {{
            return format!(\"{{{{ \\\"result\\\": {{}} }}}}\", output.unwrap());
        }}

        return \"{{ \\\"error\\\": \\\"Failed to serialize output\\\" }}\".to_owned();
    }}
"
                ));
            }
        }

        code.push_str(
            format!(
                "
    pub async fn handle(ctx: machinery::context::Context, fn_name: String, json_input: String) -> String {{
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
        self.build_rust_handler(&mut combined_result)?;

        Ok(())
    }
}

pub fn default(root_dir: &str) -> Pipeline {
    Pipeline {
        root_dir: root_dir.to_owned(),
        files: Vec::new(),
        export_dir: None,
        base_crate_path: "crate".to_owned(),
        debug_comments: false,
        introspection: false,
        custom_types: vec![],
        custom_header: None,
        custom_footer: None,
    }
}
