mod ast_parser;
mod constant;
mod files;
mod graph;
mod packages;
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
    println!("package_files(path) took {}ms", sw.elapsed_ms());

    sw.start();
    let packages = packages::build(package_files);
    println!("packages::build(package_files) took {}ms", sw.elapsed_ms());

    sw.start();
    let _resolved_references = reference_resolver::resolve(&packages.definitions, &packages.references);
    println!(
        "reference_resolver::resolve(&packages.definitions, packages.references) took {}ms",
        sw.elapsed_ms()
    );
}
