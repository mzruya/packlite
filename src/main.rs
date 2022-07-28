mod ast;
mod files;
mod parser;
mod validator;
use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;
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

    /// Root directory of the project
    #[clap(short, long)]
    ignore_constants: Vec<String>,
}

impl CliCommand {
    fn ignored_constants(&self) -> Vec<String> {
        let mut ignored_constants = self.ignore_constants.clone();
        let mut built_in = Self::built_in_ignored_constants();

        ignored_constants.append(&mut built_in);

        ignored_constants
    }

    fn built_in_ignored_constants() -> Vec<String> {
        vec!["String", "Devise", "Array", "RSpec", "T", "Date", "DateTime", "BigDecimal", "Rails"]
            .into_iter()
            .map(str::to_owned)
            .collect_vec()
    }
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
    let parsed_files = parser::parse_ruby_files(&command.root_dir, &ruby_files);

    debug!("parser::resolve_references()");
    let (definitions, references) = parser::resolve_references(parsed_files);

    debug!("parser::apply_package_metadata()");
    let project = parser::apply_package_metadata(definitions, references, packages, &command.public_path, &command.ignored_constants());

    debug!("validator::validate()");
    let violations = validator::validate(&project);
    debug!("found {} violations", violations.len());

    debug!("validator::validate()");
    let deprecated_references = validator::deprecated_references(&violations);
    debug!("found {} violations", violations.len());

    debug!("packages::log()");
    std::fs::File::create("/users/matan.zruya/desktop/project.json")
        .unwrap()
        .write_all(serde_json::to_string_pretty(&project).unwrap().as_bytes())
        .unwrap();

    std::fs::File::create("/users/matan.zruya/desktop/validations.json")
        .unwrap()
        .write_all(serde_json::to_string_pretty(&violations).unwrap().as_bytes())
        .unwrap();

    for deprecated_reference in deprecated_references {
        let package = project.packages.iter().find(|package| package.name == deprecated_reference.violating_pack).unwrap();
        let deprecated_reference_file_path = package.root.join("deprecated_references.yml");

        std::fs::File::create(deprecated_reference_file_path)
            .unwrap()
            .write_all(serde_yaml::to_string(&deprecated_reference.deprecated_references).unwrap().as_bytes())
            .unwrap();
    }
    debug!("end");
}
