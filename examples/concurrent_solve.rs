use russcip::prelude::*;

fn main() {
    // Pass "deterministic" as the first argument to solve in deterministic mode;
    // anything else (or no argument) uses the default opportunistic mode.
    let deterministic = std::env::args().nth(1).as_deref() == Some("deterministic");

    // Read the MIPLIB instance p0201.
    let model = Model::new()
        .include_default_plugins()
        .read_prob("data/test/p0201.mps")
        .unwrap();

    // Use one thread per available CPU core for the concurrent solve. The
    // `bundled` SCIP library is built with thread support, so this runs several
    // SCIP solvers in parallel and returns the best result.
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as i32;
    let mode = if deterministic { 1 } else { 0 };
    println!(
        "Solving with {n_threads} threads in {} mode",
        if deterministic {
            "deterministic"
        } else {
            "opportunistic"
        }
    );
    let solved_model = model
        .set_int_param("parallel/maxnthreads", n_threads)
        .unwrap()
        .set_int_param("parallel/mode", mode)
        .unwrap()
        .solve_concurrent();

    let status = solved_model.status();
    println!("Solved with status {status:?}");

    let obj_val = solved_model.obj_val();
    println!("Objective value: {obj_val}");
}
