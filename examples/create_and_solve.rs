use russcip::prelude::*;

fn main() {
    // Create model
    let mut model = Model::default().maximize();

    // Add variables
    let x1 = model.add(var().integer(0, isize::MAX).obj(3.).name("x1"));
    let x2 = model.add(var().integer(0, isize::MAX).obj(2.).name("x2"));

    // Add constraints
    model.add(cons().name("c1").coef(&x1, 2.).coef(&x2, 1.).le(100.));
    model.add(cons().name("c2").coef(&x1, 1.).coef(&x2, 2.).le(80.));

    let solved_model = model.solve();

    let status = solved_model.status();
    println!("Solved with status {:?}", status);

    let obj_val = solved_model.obj_val();
    println!("Objective value: {}", obj_val);

    let sol = solved_model.best_sol().unwrap();
    let vars = solved_model.vars();

    for var in vars {
        println!("{} = {}", var.name(), sol.val(&var));
    }
}
