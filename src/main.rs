use russcip::model::Model;

fn main() {
    let scip = Model::new().unwrap();
    scip.print_version().unwrap();
    
    let path = std::env::args()
        .nth(1)
        .expect("Expected filename of lp file as an argument");

    let mut model = Model::new().unwrap();
    model.include_default_plugins().unwrap();
    model.read_prob(&path).unwrap();
    // model.set_real_param("limits/time", 1.).unwrap();
    model.solve().unwrap();
    let sol = model.get_best_sol().unwrap();

    let status = model.get_status();
    println!("Status: {:?}", status);
    println!("Obj val: {}", model.get_obj_val());
    println!("N vars: {}", model.get_n_vars());
    println!("Best solution found: {:?}", sol);
}
