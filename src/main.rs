mod ast_parser;
mod constant;
mod files;
mod packages;
mod reference_graph;
mod reference_resolver;
use tracing::debug;

fn main() {
    install_logger();

    do_run("/Users/matan.zruya/workspace/gusto/zenpayroll/packs");
}

fn install_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_writer(std::io::stderr)
        .init();
}

fn do_run(path: &str) {
    // Lists all the `.rb` and `package.yml` files inside a folder
    let file_paths = files::all(path);
    debug!("files::all(path)");

    // Groups ruby file paths into packages, each package includes the ruby constant references and definitions.
    let packages = packages::build(file_paths);
    debug!("packages::build(package_files)");

    // Resolves ruby constant definitions to the fully qualified constant they refer to.
    let resolved_references = reference_resolver::resolve(&packages.definitions, &packages.references);
    debug!("reference_resolver::resolve(&packages.definitions, packages.references)",);

    // Indexes all the references and definitions into a graph data structure.
    let reference_graph = reference_graph::build_reference_graph(packages.definitions, resolved_references);
    debug!("graph::build(&packages.definitions, &resolved_references)",);

    let usages = reference_graph.find_usages("Pufferfish::ValueProviders::Company");
    debug!("graph.find_usages()");

    // println!("{usages:#?}");
}
