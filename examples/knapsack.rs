use russcip::minimal_model;
use russcip::prelude::*;

/// 0-1 Knapsack problem
#[derive(Debug)]
struct Knapsack {
    /// Sizes of the items
    sizes: Vec<usize>,
    /// Values of the items
    values: Vec<usize>,
    /// Capacity of the knapsack
    capacity: usize,
}

/// Solution to the knapsack problem
#[derive(Debug)]
struct KnapsackSolution {
    /// Indices of the items in the solution
    items: Vec<usize>,
    /// Total value of the solution
    value: f64,
}

impl Knapsack {
    fn new(sizes: Vec<usize>, values: Vec<usize>, capacity: usize) -> Self {
        assert_eq!(
            sizes.len(),
            values.len(),
            "Sizes and values must have the same length"
        );
        Knapsack {
            sizes,
            values,
            capacity,
        }
    }

    /// Solves the 0-1 knapsack as an integer program
    fn solve(&self) -> KnapsackSolution {
        let mut model = minimal_model().maximize();

        let mut vars = Vec::with_capacity(self.sizes.len());
        for i in 0..self.sizes.len() {
            vars.push(model.add(var().binary().obj(self.values[i] as f64)));
        }

        let mut capacity_cons = cons().le(self.capacity as f64);
        for (i, var) in vars.iter().enumerate() {
            capacity_cons = capacity_cons.coef(var, self.sizes[i] as f64);
        }
        model.add(capacity_cons);

        let solved_model = model.solve();

        let sol = solved_model.best_sol().unwrap();
        let mut items = vec![];
        for (i, var) in vars.iter().enumerate() {
            if sol.val(var) > 0.5 {
                items.push(i);
            }
        }
        let value = sol.obj_val();
        KnapsackSolution { items, value }
    }
}

fn main() {
    let knapsack = Knapsack::new(vec![2, 3, 4, 5], vec![3, 4, 5, 6], 6);
    let solution = knapsack.solve();

    println!("Input: {:?}", knapsack);
    println!("Solution items: {:?}", solution.items);
    println!(
        "Value: {} = {:?}",
        solution.value,
        solution
            .items
            .iter()
            .map(|&i| (knapsack.sizes[i], knapsack.values[i]))
            .collect::<Vec<_>>()
    );
}
