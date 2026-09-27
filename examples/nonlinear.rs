//! Nonlinear constraints: the `cons!` macro, comprehensions, and how to handle
//! a nonlinear objective.
//!
//! Run with `cargo run --example nonlinear`.

use russcip::prelude::*;

/// Rosenbrock: minimise `(1-x)^2 + 100*(y - x^2)^2`, whose optimum is 0 at (1,1).
///
/// SCIP has no nonlinear objective, so the usual trick is an auxiliary variable
/// `t` that is minimised subject to `expression <= t`.
fn rosenbrock() {
    let mut model = Model::default().minimize().hide_output();

    let x = model.add(var().name("x").cont(-2.0..=2.0));
    let y = model.add(var().name("y").cont(-2.0..=2.0));
    let t = model.add(var().name("t").obj(1.0).cont(0.0..=1e6));

    // Note `^` binds tighter than `*` here, as in mathematics — unlike Rust's
    // own `^`, which is bitwise xor and binds looser than `+`.
    model.add(cons!((1 - x) ^ 2 + 100 * (y - x ^ 2) ^ 2 <= t).name("rosenbrock"));

    let solved = model.solve();
    let sol = solved.best_sol().unwrap();

    println!("Rosenbrock");
    println!("  status    = {:?}", solved.status());
    println!("  objective = {:.6}", solved.obj_val());
    println!("  x = {:.4}, y = {:.4}", sol.val(&x), sol.val(&y));
    println!();
}

/// A norm-ball constrained problem, showing comprehensions: maximise a weighted
/// sum subject to `sum(x_i^2) <= r^2`.
fn ball_constrained() {
    const N: usize = 5;
    let radius: f64 = 3.0;
    let profit = [1.0, 2.0, 3.0, 2.5, 0.5];

    let mut model = Model::default().maximize().hide_output();

    let xs: Vec<_> = (0..N)
        .map(|i| model.add(var().name(&format!("x{i}")).cont(0.0..=10.0)))
        .collect();

    let obj = model.add(var().name("obj").obj(1.0).cont(0.0..=1e6));
    model.add(cons!(sum(i in 0..N, profit[i] * xs[i]) = obj));

    model.add(cons!(sum(x in &xs, x ^ 2) <= radius * radius).name("ball"));

    let solved = model.solve();
    let sol = solved.best_sol().unwrap();

    println!("Weighted sum over a ball of radius {radius}");
    println!("  status    = {:?}", solved.status());
    println!("  objective = {:.6}", solved.obj_val());
    for (i, x) in xs.iter().enumerate() {
        println!("  x{i} = {:.4}", sol.val(x));
    }
    println!();
}

/// `signpower(x, n) = sign(x)*|x|^n` — the odd extension of a power, used where
/// a quantity is direction-dependent (pressure/flow relations, for instance).
/// Unlike `x^n` it is defined for negative `x` at fractional exponents.
fn signed_power() {
    let mut model = Model::default().hide_output();
    let q = model.add(var().name("q").cont(-4.0..=4.0));

    // signpower(q, 1.5) == -8  =>  |q|^1.5 = 8 with q negative  =>  q = -4
    model.add(cons!(signpower(q, 1.5) = -8));

    let solved = model.solve();
    let sol = solved.best_sol().unwrap();

    println!("Signed power");
    println!("  status = {:?}", solved.status());
    println!(
        "  q      = {:.4}  (a plain q^1.5 is undefined here)",
        sol.val(&q)
    );
}

fn main() {
    rosenbrock();
    ball_constrained();
    signed_power();
}
