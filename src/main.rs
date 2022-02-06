mod ast;
mod dependency_graph;
mod files;
mod package_validator;
mod packages;
use std::path::{Path, PathBuf};

use clap::Parser;
use tracing::debug;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliCommand {
    path: PathBuf,
}

fn main() {
    install_logger();

    let command = CliCommand::parse();
    do_run(&command.path)
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

fn do_run(project_root: &Path) {
    // Walk the directory tree and find all ruby source files grouped by their respective packages
    debug!("files::all()");
    let packages = files::all(project_root);
    debug!(
        "{} packages, with {} total ruby files",
        packages.len(),
        packages.iter().map(|p| p.ruby_files.len()).sum::<usize>()
    );

    // Convert file paths to actual data, by parsing the ast, doing reference lookups and the whole shebang
    debug!("packages::parse()");
    let packages = packages::parse(packages);

    // Indexes all the references and definitions into a graph data structure.
    debug!("graph::build()",);
    let dependency_graph = dependency_graph::build(&packages);

    debug!("package_validator::build()");
    let package_validator = package_validator::build(dependency_graph);

    println!("Found {} violations", package_validator.validate_all().len());
}
