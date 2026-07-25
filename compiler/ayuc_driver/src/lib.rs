pub(crate) mod context;

use std::{
    collections::{HashMap, VecDeque},
    env,
    fs::{self},
    path::{Path, PathBuf},
    process::ExitCode,
};

use ayuc_diagnostic::DiagnosticContext;
use ayuc_id::ModuleId;
use ayuc_lexer::{LexedFile, stream::TokenStream};
use ayuc_parser::Parser;
use ayuc_source::SourceCache;

use crate::context::CompilerContext;

fn print_diagnostics(dcx: &DiagnosticContext, source_cache: &SourceCache) {
    for advice in dcx.advice() {
        let _ = advice.to_ariadne().eprint(source_cache);
    }

    for warning in dcx.warnings() {
        let _ = warning.to_ariadne().eprint(source_cache);
    }

    for error in dcx.errors() {
        let _ = error.to_ariadne().eprint(source_cache);
    }
}

/// Topological sort for all modules to determine the order for compilation.
fn get_compilation_order(ctx: &mut CompilerContext) -> Option<Vec<ModuleId>> {
    let module_count = ctx.module_registry.trees.len();

    let mut in_degrees: HashMap<ModuleId, usize> = HashMap::with_capacity(module_count);
    let mut dependents: HashMap<ModuleId, Vec<ModuleId>> = HashMap::with_capacity(module_count);

    for module_id in ctx.module_registry.trees.keys() {
        in_degrees.insert(module_id, 0);
    }

    for (module_id, dependencies) in &ctx.module_registry.dependencies {
        in_degrees.insert(module_id, dependencies.len());

        for (_, dep_id) in dependencies {
            dependents.entry(*dep_id).or_default().push(module_id);
        }
    }

    let mut no_edges: VecDeque<ModuleId> = in_degrees
        .iter()
        .filter_map(|(&id, &degree)| if degree == 0 { Some(id) } else { None })
        .collect();

    let mut compilation_order = Vec::with_capacity(module_count);

    while let Some(ready_module) = no_edges.pop_front() {
        compilation_order.push(ready_module);

        if let Some(waiting_modules) = dependents.get(&ready_module) {
            for waiting_mod in waiting_modules {
                if let Some(degree) = in_degrees.get_mut(waiting_mod) {
                    *degree -= 1;

                    if *degree == 0 {
                        no_edges.push_back(*waiting_mod);
                    }
                }
            }
        }
    }

    if compilation_order.len() == module_count {
        Some(compilation_order)
    } else {
        None
    }
}

pub fn parse_file(ctx: &mut CompilerContext, path: PathBuf) -> ModuleId {
    let path = path.canonicalize().expect("unable to canonicalize");
    let file_path = path.to_str().expect("invalid path");

    assert!(!path.is_dir());

    let ast = {
        let file_content = fs::read_to_string(file_path).expect("unable to read file");
        let file_id = ctx.source_cache.add(file_path, file_content);
        let source = ctx
            .source_cache
            .source_of(file_id)
            .expect("file_id from .add is inaccessible");

        if let Some(LexedFile { tokens }) = ayuc_lexer::lex(&mut ctx.dcx, file_id, source.text()) {
            let parser = Parser::new(
                &mut ctx.dcx,
                file_id,
                source.text(),
                TokenStream::new(&tokens),
                &mut ctx.sess,
            );

            parser.parse_full()
        } else {
            print_diagnostics(&ctx.dcx, &ctx.source_cache);

            None
        }
    };

    if let Some(ast) = ast {
        ctx.module_registry.add_ast(file_path.to_string(), ast)
    } else {
        ctx.module_registry.add_failed_ast(file_path.to_string())
    }
}

pub fn drive() -> ExitCode {
    let mut ctx = CompilerContext::default();

    let args = env::args().skip(1).collect::<Vec<_>>();
    let input_file = match args.first() {
        Some(input_file) => Path::new(input_file).to_path_buf(),
        _ => panic!("no file provided"),
    };

    let mut to_parse = vec![(None, input_file)];

    while let Some((dependency_of, file_path)) = to_parse.pop() {
        let working_directory = file_path
            .parent()
            .expect("no parent directory")
            .to_path_buf();

        let absolute = file_path.to_str().expect("invalid path");
        let module_id = if let Some(id) = ctx.module_registry.id_by_path.get(absolute) {
            *id
        } else {
            parse_file(&mut ctx, file_path)
        };

        if let Some(ast) = &ctx.module_registry.trees[module_id] {
            let file_modules = ast
                .items
                .iter()
                .flat_map(|i| match &i.kind {
                    ayuc_ast::ItemKind::FileMod(file_module) => {
                        Some((i.id, file_module.name.sym.as_str()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            for (node_id, required_module) in file_modules {
                to_parse.push((
                    Some((node_id, module_id)),
                    working_directory.join(format!("{}.ayu", required_module)),
                ));
            }
        }

        if let Some((node_id, origin_id)) = dependency_of {
            ctx.module_registry
                .dependencies
                .entry(origin_id)
                .unwrap()
                .and_modify(|list| list.push((node_id, module_id)))
                .or_insert(vec![(node_id, module_id)]);
        }
    }

    let Some(compilation_order) = get_compilation_order(&mut ctx) else {
        println!("circular dependency somewhere");

        return ExitCode::FAILURE;
    };

    /*let output = args
        .get(1)
        .and_then(|name| Path::new(name).file_name())
        .and_then(|o| o.to_str());

    let source = source_cache
        .source_of(file_id)
        .expect("inaccessible source")
        .text();

    eprintln!(
        "> Compiling {}",
        source_cache.name_of(file_id).expect("inaccessible source")
    );

    let mut dcx = DiagnosticContext::new();

    let Some(LexedFile { tokens }) = ayuc_lexer::lex(&mut dcx, file_id, source) else {
        print_diagnostics(dcx, &source_cache);

        return ExitCode::FAILURE;
    };

    let parser = Parser::new(
        &mut dcx,
        file_id,
        source,
        TokenStream::new(&tokens),
        &mut sess,
    );
    let ast = parser.parse_full();

    if ast.is_none() || dcx.requires_abort() {
        let errors = dcx.errors().len();

        print_diagnostics(dcx, &source_cache);

        eprintln!(
            "> Unable to compile due to {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        );

        return ExitCode::FAILURE;
    }

    let ast = ast.unwrap();
    let required_file_modules = ast
        .items
        .iter()
        .flat_map(|i| {
            if let ayuc_ast::ItemKind::FileMod(file_module) = &i.kind {
                Some(format!("{}.ayu", file_module.name.sym))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    println!("File requires following files: {:?}", required_file_modules);

    let rcx = Resolver::resolve(&mut sess, &mut dcx, file_id, &ast);

    if dcx.requires_abort() {
        let errors = dcx.errors().len();

        print_diagnostics(dcx, &source_cache);

        eprintln!(
            "> Unable to compile due to {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        );

        return ExitCode::FAILURE;
    }

    SemanticAnalyzer::analyze(&ast, file_id, &rcx, &mut dcx, &sess);

    if !dcx.errors().is_empty() {
        let errors = dcx.errors().len();

        print_diagnostics(dcx, &source_cache);

        eprintln!(
            "> Unable to compile due to {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        );

        return ExitCode::FAILURE;
    }

    let lowering = AstLowering::new(&rcx);
    let lcx = lowering.lower(&ast);
    let code = LuauCodegen::emit(&rcx, &lcx, &sess);

    if let Some(output) = output
        && let Ok(mut file) = File::create(output)
    {
        file.write_all(code.as_bytes())
            .expect("unable to write to file");
    } else {
        println!("{code}");
    }*/

    ExitCode::SUCCESS
}
