mod ast_parser;
mod constant;
mod files;
mod packages;
mod reference_graph;
mod reference_resolver;
use stopwatch::Stopwatch;

fn main() {
    // let pool = rayon::ThreadPoolBuilder::new().num_threads(16).build().unwrap();
    // pool.install(|| {
    //     do_run("/Users/matan.zruya/workspace/gusto/zenpayroll/packs");
    // });

    do_run("/Users/matan.zruya/workspace/gusto/zenpayroll/packs");
}

fn do_run(path: &str) {
    let mut sw = Stopwatch::start_new();

    sw.start();
    let package_files = files::all(path);
    println!("files::all(path) took {}ms", sw.elapsed_ms());

    sw.start();
    let packages = packages::build(package_files);
    println!("packages::build(package_files) took {}ms", sw.elapsed_ms());

    sw.start();
    let resolved_references = reference_resolver::resolve(&packages.definitions, &packages.references);
    println!(
        "reference_resolver::resolve(&packages.definitions, packages.references) took {}ms",
        sw.elapsed_ms()
    );

    sw.start();
    let reference_graph = reference_graph::build_reference_graph(packages.definitions, resolved_references);
    println!(
        "graph::build(&packages.definitions, &resolved_references) took {}ms",
        sw.elapsed_ms()
    );

    sw.start();
    let usages = reference_graph.find_usages("Pufferfish::ValueProviders::Company");
    println!("graph.find_usages took {}ms", sw.elapsed_ms());

    println!("{usages:#?}");
}
