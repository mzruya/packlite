mod ast;
mod files;
mod parser;
mod resolver;
mod validator;
use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;
use std::io::prelude::*;
use tracing::{debug, instrument};

#[derive(clap::Args, Debug)]
struct UpdateDeprecations {
    pack: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    UpdateDeprecations(UpdateDeprecations),
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliCommand {
    #[clap(subcommand)]
    command: Command,

    /// Root directory of the project
    #[clap(short, long, default_value = ".")]
    root_dir: PathBuf,

    /// Where a package defines its public api
    #[clap(short, long, default_value = "app/public")]
    public_path: String,

    /// paths to scan for packages
    #[clap(long)]
    package_paths: Vec<PathBuf>,

    /// constants that we should omit from reference resolution
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

#[instrument(skip_all)]
fn do_run(command: CliCommand) {
    debug!("reading file paths");
    let ruby_files = files::all_ruby_files(&command.root_dir, &command.package_paths);
    let packages = files::all_packages(&command.root_dir, &command.package_paths);
    debug!("found {} packages and {} ruby files", packages.len(), ruby_files.len());

    debug!("parsing ruby files");
    let parsed_files = parser::parse_ruby_files(&command.root_dir, &ruby_files);

    debug!("resolving references");
    let (definitions, references) = resolver::resolve_references(parsed_files);
    let project = parser::apply_package_metadata(definitions, references, packages, &command.public_path, &command.ignored_constants());

    std::fs::File::create("/users/matan.zruya/desktop/output.json")
        .unwrap()
        .write_all(serde_json::to_string_pretty(&project).unwrap().as_bytes())
        .unwrap();

    debug!("running {:?}", command.command);
    match command.command {
        Command::UpdateDeprecations(_cmd) => {}
    }
}

fn update_deprecations(command: &UpdateDeprecations, project: &parser::Project) {
    let violations = validator::validate(project);
    let mut deprecated_references = validator::deprecated_references(&violations);

    if let Some(pack) = &command.pack {
        deprecated_references = deprecated_references
            .into_iter()
            .filter(|deprecated_reference| &deprecated_reference.violating_pack == pack)
            .collect_vec();
    }

    for deprecated_reference in deprecated_references {
        let package = project.packages.iter().find(|package| package.name == deprecated_reference.violating_pack).unwrap();
        let deprecated_reference_file_path = package.root.join("deprecated_references.yml");

        std::fs::File::create(deprecated_reference_file_path)
            .unwrap()
            .write_all(serde_yaml::to_string(&deprecated_reference.deprecated_references).unwrap().as_bytes())
            .unwrap();
    }
}
