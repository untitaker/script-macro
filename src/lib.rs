#![doc = include_str!("../README.md")]

extern crate proc_macro;

use std::fmt::{Debug};
use std::path::PathBuf;

use proc_macro::TokenStream;
use rhai::{Engine, Scope, packages::Package, ImmutableString, Dynamic, EvalAltResult, Position};
use rhai_fs::FilesystemPackage;
use syn::{parse_macro_input, LitStr, parse::{ParseStream, Parse}};

struct RunScriptInput {
    script_source: LitStr,
}

impl Parse for RunScriptInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            script_source: input.parse()?,
        })
    }
}

fn get_source_context(source_code: &str, padding: usize, pos: Position) -> String {
    let mut source_snippet = String::new();

    if let Some(lineno) = pos.line() {
        let lines: Vec<_> = source_code.split('\n').collect();
        for (i, line) in lines[
            (lineno - padding).clamp(0, lines.len())
            ..
            (lineno + padding).clamp(0, lines.len())
        ].iter().enumerate() {
            if i == padding - 1 {
                source_snippet.push_str("--> ");
            } else {
                source_snippet.push_str("    ");
            }

            source_snippet.push_str(line);
            source_snippet.push('\n');
        }
    }

    source_snippet
}

fn handle_runtime_error(source_code: &str, e: Box<EvalAltResult>) {
    let pos = {
        let mut inner_error = &e;

        while let EvalAltResult::ErrorInModule(_, err, ..)
            | EvalAltResult::ErrorInFunctionCall(_, _, err, ..) = &**inner_error {
            inner_error = err;
        }

        inner_error.position()
    };

    panic!("{}\n\n{}", e, get_source_context(source_code, 3, pos));
}

#[proc_macro]
pub fn run_script(params: TokenStream) -> TokenStream {
    let args = parse_macro_input!(params as RunScriptInput);

    let engine = get_default_engine();
    let output: String = engine.eval(&args.script_source.value()).map_err(|e| handle_runtime_error(&args.script_source.value(), e)).unwrap();

    output.parse().expect("invalid token stream")
}

#[proc_macro_attribute]
pub fn run_script_on(params: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(params as RunScriptInput);
    let engine = get_default_engine();

    let mut scope = Scope::new();
    scope.push("item", item.to_string());
    let output: String = engine.eval_with_scope(&mut scope, &args.script_source.value()).map_err(|e| handle_runtime_error(&args.script_source.value(), e)).unwrap();

    output.parse().expect("invalid token stream")
}

fn get_default_engine() -> Engine {
    let mut engine = Engine::new();

    engine.set_max_expr_depths(100, 100);

    engine.register_fn("parse_yaml", helper_parse_yaml);
    engine.register_fn("parse_json", helper_parse_json);
    engine.register_fn("stringify_yaml", helper_stringify_yaml);
    engine.register_fn("stringify_json", helper_stringify_json);
    engine.register_fn("slugify_ident", helper_slugify_ident);
    engine.register_fn("glob", helper_glob);
    engine.register_fn("basename", helper_basename);

    let package = FilesystemPackage::new();
    package.register_into_engine(&mut engine);

    engine
}

fn coerce_err(x: impl Debug) -> Box<EvalAltResult> {
    format!("{x:?}").into()
}

fn helper_parse_yaml(input: ImmutableString) -> Result<Dynamic, Box<EvalAltResult>> {
    serde_yaml::from_str(input.as_str()).map_err(coerce_err)
}

fn helper_stringify_yaml(input: Dynamic) -> Result<ImmutableString, Box<EvalAltResult>> {
    serde_yaml::to_string(&input).map(From::from).map_err(coerce_err)
}

fn helper_parse_json(input: ImmutableString) -> Result<Dynamic, Box<EvalAltResult>> {
    serde_json::from_str(input.as_str()).map_err(coerce_err)
}

fn helper_stringify_json(input: Dynamic) -> Result<ImmutableString, Box<EvalAltResult>> {
    serde_json::to_string(&input).map(From::from).map_err(coerce_err)
}


fn helper_slugify_ident(input: ImmutableString) -> ImmutableString {
    let mut is_first_char = true;
    input.as_str().replace(|x: char| {
        if is_first_char && x.is_ascii_digit() {
            return true;
        }
        is_first_char = false;

        !matches!(x, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    }, "_").into()
}

fn helper_glob(pattern: ImmutableString) -> Result<Dynamic, Box<EvalAltResult>> {
    let mut result = Vec::new();

    for entry in glob::glob(pattern.as_str()).map_err(coerce_err)? {
        let entry = entry.map_err(coerce_err)?;

        result.push(entry);
    }

    Ok(result.into())
}

fn helper_basename(input: PathBuf) -> Result<ImmutableString, Box<EvalAltResult>> {
    Ok(input.file_name().unwrap_or(input.as_os_str()).to_str().ok_or("basename is not valid unicode")?.to_owned().into())
}
