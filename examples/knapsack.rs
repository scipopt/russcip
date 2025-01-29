use russcip::minimal_model;
use russcip::prelude::*;

#[derive(Debug)]
struct Knapsack {
    sizes: Vec<usize>,
    values: Vec<usize>,
    capacity: usize,
}

#[derive(Debug)]
struct KnapsackSolution {
    items: Vec<usize>,
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

        let vars: Vec<_> = self
            .values
            .iter()
            .map(|&value| model.add(var().binary().obj(value as f64)))
            .collect();

        let var_sizes = vars
            .iter()
            .zip(self.sizes.iter())
            .map(|(var, &size)| (var, size as f64));
        model.add(cons().le(self.capacity as f64).expr(var_sizes));

        let solved_model = model.solve();

        let sol = solved_model.best_sol().unwrap();
        let items: Vec<_> = vars
            .iter()
            .enumerate()
            .filter(|(_, v)| sol.val(v) > 1e-6)
            .map(|(i, _)| i)
            .collect();
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
