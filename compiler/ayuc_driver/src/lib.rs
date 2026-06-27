use std::{
    env,
    fs::{self},
    path::Path,
    process::ExitCode,
};

use ayuc_codegen::LuauCodegen;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_lexer::{LexedFile, stream::TokenStream};
use ayuc_lower::AstLowering;
use ayuc_parser::Parser;
use ayuc_resolve::Resolver;
use ayuc_source::SourceCache;
use ayuc_tyctx::TyCtx;

fn print_diagnostics(dcx: DiagnosticContext, source_cache: &SourceCache) {
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

pub fn drive() -> ExitCode {
    let mut source_cache = SourceCache::default();

    let args = env::args().skip(1).collect::<Vec<_>>();

    let file_id = match args.first() {
        Some(input_file) => {
            let path = Path::new(&input_file);
            let content = fs::read_to_string(path).expect("unable to read file");

            source_cache.add(
                path.canonicalize()
                    .expect("unable to canonicalize")
                    .to_str()
                    .expect("unable to canonicalize"),
                content,
            )
        }
        _ => panic!("no file provided"),
    };

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

    let parser = Parser::new(&mut dcx, file_id, source, TokenStream::new(&tokens));
    let ast = parser.parse_full();

    if ast.is_none() || !dcx.errors().is_empty() {
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

    let mut ty_ctx = TyCtx {
        packages: Vec::new(),
        next_package_id: 0,
    };

    let resolver = Resolver::resolve(&ast);

    let lowering = AstLowering::new(&mut ty_ctx, &resolver);
    let package = lowering.lower(&ast);
    let package_id = ty_ctx.register_package(package);

    println!();
    println!("{}", LuauCodegen::emit(package_id, &ty_ctx));

    ExitCode::SUCCESS
}
