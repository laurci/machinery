use syn::{visit::Visit, Attribute};

#[derive(Debug)]
pub enum MessageKind {
    Enum,
    Struct,
}

#[derive(Debug)]
pub struct Message {
    pub kind: MessageKind,
    pub name: String,
    pub location: String,
    pub code: String,
}

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub location: String,
    pub arguments: Vec<String>,
    pub return_type: String,
}

#[derive(Debug)]
pub struct MachineryVisitor {
    file_location: String,
    services: Vec<Service>,
    messages: Vec<Message>,
}

#[derive(Debug)]
pub struct AnalyzeResult {
    pub file_location: String,
    pub services: Vec<Service>,
    pub messages: Vec<Message>,
}

fn has_machinery_attribute(attrs: Vec<Attribute>, name: &str) -> bool {
    let full_name = format!("machinery::{}", name);

    for attr in attrs {
        let path = attr
            .path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        if path == full_name {
            return true;
        }
    }

    false
}

fn parse_enum(item: &syn::ItemEnum) -> String {
    let mut output = String::new();

    output.push_str(&format!("export type {} = \n", item.ident.to_string())[..]);

    for variant in item.variants.iter() {
        let name = variant.ident.to_string();

        output.push_str(&format!("\t\"{}\"", name));

        if name == item.variants.last().unwrap().ident.to_string() {
            output.push_str(";\n");
        } else {
            output.push_str(" |\n");
        }
    }

    if item.variants.is_empty() {
        output.push_str("never;\n");
    }

    output
}

fn parse_struct(item: &syn::ItemStruct) -> String {
    let mut output = String::new();

    output.push_str(&format!("export interface {} {{\n", item.ident.to_string()));

    for member in item.fields.iter() {
        if member.ident.is_none() {
            continue;
        }
        let name = member.ident.clone().unwrap().to_string();

        let ty = &member.ty;
        let ty = quote::quote!(#ty).to_string();

        output.push_str(&format!("\t{}: {},\n", name, ty));
    }

    output.push_str("}\n");

    output
}

fn parse_fn_arguments(item: &syn::ItemFn) -> Vec<String> {
    let mut arguments = Vec::new();

    for input in item.sig.inputs.iter() {
        match input {
            syn::FnArg::Typed(pat) => {
                let ident = pat.pat.as_ref();
                let name = quote::quote!(#ident).to_string();

                let ty = &pat.ty;
                let ty = quote::quote!(#ty).to_string();

                arguments.push(format!("{}: {}", name, ty));
            }
            _ => continue,
        }
    }

    arguments
}

fn parse_fn_return_type(item: &syn::ItemFn) -> String {
    match item.sig.output {
        syn::ReturnType::Default => return "void".to_string(),
        syn::ReturnType::Type(_, ref ty) => {
            let ty = &ty;
            let ty = quote::quote!(#ty).to_string();

            ty
        }
    }
}

impl<'ast> Visit<'ast> for MachineryVisitor {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        if !has_machinery_attribute(i.attrs.clone(), "service") {
            return;
        }

        let name = i.sig.ident.to_string();

        let arguments = parse_fn_arguments(i);
        let return_type = parse_fn_return_type(i);

        self.services.push(Service {
            name,
            location: self.file_location.clone(),
            arguments,
            return_type,
        });
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if !has_machinery_attribute(i.attrs.clone(), "message") {
            return;
        }

        let name = i.ident.to_string();

        self.messages.push(Message {
            kind: MessageKind::Struct,
            name,
            location: self.file_location.clone(),
            code: parse_struct(i),
        });
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if !has_machinery_attribute(i.attrs.clone(), "message") {
            return;
        }

        let name = i.ident.to_string();

        self.messages.push(Message {
            kind: MessageKind::Enum,
            name,
            location: self.file_location.clone(),
            code: parse_enum(i),
        });
    }
}

pub fn analyze_file(file: &syn::File, path: &str) -> AnalyzeResult {
    let file_location = path
        .to_owned()
        .split("/")
        .filter(|s| s.to_owned() != ".")
        .map(|s| match s {
            "src" => "crate".to_owned(),
            _ => s.replace(".rs", ""),
        })
        .collect::<Vec<_>>()
        .join("::");

    let mut visitor = MachineryVisitor {
        file_location,
        services: Vec::new(),
        messages: Vec::new(),
    };

    visitor.visit_file(file);

    AnalyzeResult {
        file_location: visitor.file_location,
        services: visitor.services,
        messages: visitor.messages,
    }
}
