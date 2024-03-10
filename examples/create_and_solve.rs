use russcip::prelude::*;

fn main() {
    // Create model
    let mut model = Model::new()
        .hide_output()
        .include_default_plugins()
        .create_prob("test")
        .set_obj_sense(ObjSense::Maximize);

    // Add variables
    let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
    let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);

    // Add constraints
    model.add_cons(
        vec![x1.clone(), x2.clone()],
        &[2., 1.],
        -f64::INFINITY,
        100.,
        "c1",
    );
    model.add_cons(
        vec![x1.clone(), x2.clone()],
        &[1., 2.],
        -f64::INFINITY,
        80.,
        "c2",
    );

    let solved_model = model.solve();

    let status = solved_model.status();
    println!("Solved with status {:?}", status);

    let obj_val = solved_model.obj_val();
    println!("Objective value: {}", obj_val);

    let sol = solved_model.best_sol().unwrap();
    let vars = solved_model.vars();

    for var in vars {
        println!("{} = {}", &var.name(), sol.val(var));
    }
}
