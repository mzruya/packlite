mod ast;
mod files;
mod parser;
use std::path::PathBuf;

use clap::Parser;
use tracing::debug;

use std::io::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliCommand {
    /// Root directory of the project
    #[clap(short, long)]
    root_dir: PathBuf,

    /// Where a package defines its public api
    #[clap(short, long, default_value = "app/public")]
    public_path: String,

    /// Where a package defines its public api
    #[clap(long)]
    package_paths: Vec<PathBuf>,
}

fn main() {
    install_logger();

    let command = CliCommand::parse();
    do_run(command)
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

fn do_run(command: CliCommand) {
    debug!("files::all()");
    let ruby_files = files::all_ruby_files(&command.root_dir, &command.package_paths);
    let packages = files::all_packages(&command.root_dir, &command.package_paths);
    debug!("found {} packages and {} ruby files", packages.len(), ruby_files.len());

    // Convert file paths to actual data, by parsing the ast, doing reference lookups and the whole shebang
    debug!("parser::parse_ruby_files()");
    let parsed_files = parser::parse_ruby_files(&ruby_files);

    debug!("parser::resolve_references()");
    let (definitions, references) = parser::resolve_references(parsed_files);

    debug!("parser::apply_package_metadata()");
    let project = parser::apply_package_metadata(definitions, references, packages, &command.public_path);

    debug!("packages::parse()");
    std::fs::File::create("/users/matan.zruya/desktop/output.json")
        .unwrap()
        .write_all(serde_json::to_string_pretty(&project).unwrap().as_bytes())
        .unwrap();
    debug!("end");
}
