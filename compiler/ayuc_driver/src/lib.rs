use std::{
    env,
    fs::{self},
    path::Path,
    process::ExitCode,
};

use ayuc_codegen::LuauCodegen;
use ayuc_lexer::{LexedFile, stream::TokenStream};
use ayuc_lower::AstLowering;
use ayuc_parser::Parser;
use ayuc_resolve::Resolver;
use ayuc_source::cache::SourceCache;
use ayuc_tyctx::TyCtx;

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

    let LexedFile {
        diagnostics,
        tokens,
    } = ayuc_lexer::lex(file_id, source).expect("unable to lex");

    let mut parser = Parser::new(file_id, source, TokenStream::new(&tokens));

    parser.extend_diagnostics(diagnostics);

    let (ast, diagnostics) = parser.parse_full();
    let diagnostics = diagnostics.unwrap();

    if !diagnostics.is_empty() {
        for diagnostic in &diagnostics {
            let _ = diagnostic.eprint(&source_cache);
        }

        println!(
            "> Unable to compile because of {} diagnostics",
            diagnostics.len()
        );
    }

    if ast.is_none() || !diagnostics.is_empty() {
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
    println!("{:#?}", package);
    let package_id = ty_ctx.register_package(package);

    println!();
    println!("{}", LuauCodegen::emit(package_id, &ty_ctx));

    ExitCode::SUCCESS
}
