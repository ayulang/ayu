pub(crate) mod context;

use std::{
    collections::{HashMap, VecDeque},
    env,
    fs::{self},
    path::{Path, PathBuf},
    process::ExitCode,
};

use ayuc_codegen::LuauCodegen;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_hir::Module;
use ayuc_id::ModuleId;
use ayuc_lexer::{LexedFile, stream::TokenStream};
use ayuc_lower::AstLowering;
use ayuc_parser::Parser;
use ayuc_resolve::{ResolutionContext, Resolver};
use ayuc_sema::SemanticAnalyzer;
use ayuc_source::SourceCache;
use slotmap::SecondaryMap;

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

    let file_content = fs::read_to_string(file_path).expect("unable to read file");
    let file_id = ctx.source_cache.add(file_path, file_content);

    let ast = {
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
            None
        }
    };

    let module_id = if let Some(ast) = ast {
        ctx.module_registry.add_ast(file_path.to_string(), ast)
    } else {
        ctx.module_registry.add_failed_ast(file_path.to_string())
    };

    ctx.module_registry.file_ids.insert(module_id, file_id);

    module_id
}

fn compile(ctx: &mut CompilerContext, module: ModuleId) -> Option<(ResolutionContext, Module)> {
    let ast = ctx.module_registry.trees[module].as_ref().unwrap();
    let file_id = ctx.module_registry.file_ids[module];

    let dcx = &mut ctx.dcx;
    let sess = &mut ctx.sess;
    let source_cache = &ctx.source_cache;

    let rcx = Resolver::resolve(&ctx.module_registry, sess, dcx, file_id, ast, module);

    if dcx.requires_abort() {
        let errors = dcx.errors().len();

        print_diagnostics(dcx, source_cache);

        eprintln!(
            "> Unable to compile due to {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        );

        return None;
    }

    SemanticAnalyzer::analyze(ast, file_id, &rcx, dcx, sess);

    if !dcx.errors().is_empty() {
        let errors = dcx.errors().len();

        print_diagnostics(dcx, source_cache);

        eprintln!(
            "> Unable to compile due to {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        );

        return None;
    }

    let lowering = AstLowering::new(module, &rcx, sess);
    let module = lowering.lower(ast);

    Some((rcx, module))
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
        let module_id = if let Some(id) = ctx.module_registry.id_by_path.get_by_left(absolute) {
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

    if ctx.dcx.requires_abort() {
        let errors = ctx.dcx.errors().len();

        print_diagnostics(&ctx.dcx, &ctx.source_cache);

        eprintln!(
            "> Unable to compile due to {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        );

        return ExitCode::FAILURE;
    }

    let mut rcxs = SecondaryMap::new();

    for module_id in compilation_order {
        let path = ctx
            .module_registry
            .id_by_path
            .get_by_right(&module_id)
            .unwrap();

        eprintln!("[compiling] {path}");

        if let Some((rcx, module)) = compile(&mut ctx, module_id) {
            ctx.module_registry.modules.insert(module_id, module);
            rcxs.insert(module_id, rcx);
        } else {
            return ExitCode::FAILURE;
        }
    }

    for (id, module) in ctx.module_registry.modules {
        let code = LuauCodegen::emit(&rcxs[id], &module, &ctx.sess);

        println!(
            "[ Compilation for \"{}\" ]",
            ctx.module_registry.id_by_path.get_by_right(&id).unwrap()
        );
        println!();
        println!("{code}");
    }

    ExitCode::SUCCESS
}
