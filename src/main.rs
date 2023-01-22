use russcip::model::Model;

fn main() {
    let scip = Model::new();
    scip.print_version();
    
    let path = std::env::args()
        .nth(1)
        .expect("Expected filename of lp file as an argument");

    let mut model = Model::new();
    model.include_default_plugins();
    model.read_prob(&path);
    // model.set_real_param("limits/time", 1.);
    model.solve();
    let sol = model.get_best_sol();

    let status = model.get_status();
    println!("Status: {:?}", status);
    println!("Obj val: {}", model.get_obj_val());
    println!("N vars: {}", model.get_n_vars());
    println!("Best solution found: {:?}", sol);
}
