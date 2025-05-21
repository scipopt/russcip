use russcip::prelude::*;
use russcip::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

struct PatternForVar(HashMap<VarId, Vec<usize>>);
struct ItemToConstraint(Vec<Constraint>);

struct BinPackingInstance {
    item_sizes: Vec<f64>,
    capacity: f64,
}

type BBNodeId = usize;

#[derive(Debug, Clone, Hash, Default, Eq, PartialEq)]
struct Pair(usize, usize);

#[derive(Debug, Clone, Default)]
struct BranchingDecisions {
    together: HashSet<Pair>,
    apart: HashSet<Pair>,
}
struct BranchingDecisionMap(HashMap<BBNodeId, BranchingDecisions>);

impl Default for BranchingDecisionMap {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(1, BranchingDecisions::default());
        BranchingDecisionMap(map)
    }
}
fn main() {
    let capacity = 15.0;
    let item_sizes = &[6.0, 5.0, 4.0, 2.0, 3.0, 7.0, 5.0, 8.0, 4.0, 5.0];

    let mut model = Model::default()
        .set_presolving(ParamSetting::Off)
        .set_separating(ParamSetting::Off)
        .set_param("display/freq", 1)
        .minimize();

    model.set_data(PatternForVar(HashMap::new()));
    model.set_data(ItemToConstraint(Vec::new()));
    model.set_data(BinPackingInstance {
        item_sizes: item_sizes.to_vec(),
        capacity,
    });
    model.set_data(BranchingDecisionMap::default());

    for i in 0..item_sizes.len() {
        let item_constraint = model.add(cons().eq(1.0).modifiable(true).removable(false));
        model
            .get_data_mut::<ItemToConstraint>()
            .unwrap()
            .0
            .insert(i, item_constraint);
    }

    // attach pricer and branching rule plugins
    model.add(pricer(KnapsackPricer {}));
    model.add(branchrule(RyanFoster {}));

    let solved_model = model.solve();

    println!("\nSolution:");
    let solution = solved_model.best_sol().unwrap();
    let pattern_for_var = &solved_model.get_data::<PatternForVar>().unwrap().0;
    for var in solved_model.vars().iter() {
        let value = solution.val(var);
        if value > 1e-6 {
            let pattern = pattern_for_var.get(&var.index()).unwrap();
            println!("{:?} = {value}", pattern);
        }
    }
    
    assert!(solved_model.eq(solution.obj_val(), 4.0));
}

struct KnapsackPricer;

fn get_duals(item_constraints: &Vec<Constraint>, farkas: bool) -> Vec<f64> {
    let mut duals = vec![0.0; item_constraints.len()];
    for (item, cons) in item_constraints.iter().enumerate() {
        let c = cons
            .transformed()
            .expect("Could not get transformed constraint");

        duals[item] = if farkas {
            c.farkas_dual_sol().expect("Could not get dual solution")
        } else {
            c.dual_sol().expect("Could not get farkas solution")
        };
    }

    duals
}

impl Pricer for KnapsackPricer {
    fn generate_columns(
        &mut self,
        mut model: Model<Solving>,
        _pricer: SCIPPricer,
        farkas: bool,
    ) -> PricerResult {
        let item_constraints = model.get_data::<ItemToConstraint>().unwrap().0.clone();

        let duals = get_duals(&item_constraints, farkas);

        let instance = model.get_data::<BinPackingInstance>().unwrap();

        let branching_decisions = model
            .get_data::<BranchingDecisionMap>()
            .unwrap()
            .0
            .get(&model.focus_node().number())
            .unwrap();

        let res = solve_knapsack(
            &instance.item_sizes,
            &duals,
            instance.capacity,
            branching_decisions,
        );

        if res.is_none() {
            return PricerResult {
                state: PricerResultState::NoColumns,
                lower_bound: None,
            };
        }

        let (sol_items, sol_value) = res.unwrap();

        let obj_coef = if farkas { 0.0 } else { 1.0 };
        let redcost = obj_coef - sol_value;

        if redcost < -model.eps() {
            // println!("-- Adding new pattern {sol_items:?} with reduced cost {redcost}");
            let new_var = model.add_priced_var(
                0.0,
                f64::INFINITY,
                1.0,
                format!("{sol_items:?}").as_str(),
                VarType::Integer,
            );

            for item in sol_items.iter() {
                model.add_cons_coef(&item_constraints[*item], &new_var, 1.0);
            }

            model
                .get_data_mut::<PatternForVar>()
                .unwrap()
                .0
                .insert(new_var.index(), sol_items.clone());

            PricerResult {
                state: PricerResultState::FoundColumns,
                lower_bound: None,
            }
        } else {
            PricerResult {
                state: PricerResultState::NoColumns,
                lower_bound: None,
            }
        }
    }
}

/// Solve the knapsack problem and return the selected items and the total profit.
fn solve_knapsack(
    sizes: &Vec<f64>,
    profits: &Vec<f64>,
    capacity: f64,
    branching_decision: &BranchingDecisions,
) -> Option<(Vec<usize>, f64)> {
    let mut model = Model::default().hide_output().maximize();

    let mut vars = Vec::with_capacity(sizes.len());
    for profit in profits {
        vars.push(model.add(var().bin().obj(*profit)));
    }

    let mut capacity_cons = cons().le(capacity);
    for (i, var) in vars.iter().enumerate() {
        capacity_cons = capacity_cons.coef(var, sizes[i]);
    }
    model.add(capacity_cons);

    // add branching decisions
    // together constraints
    for pair in branching_decision.together.iter() {
        let var1 = &vars[pair.0];
        let var2 = &vars[pair.1];
        model.add(cons().eq(1.0).coef(var1, 1.0).coef(var2, -1.0));
    }

    // apart constraints
    for pair in branching_decision.apart.iter() {
        let var1 = &vars[pair.0];
        let var2 = &vars[pair.1];
        model.add(cons().le(1.0).coef(var1, 1.0).coef(var2, 1.0));
    }

    let solved_model = model.solve();

    let sol = solved_model.best_sol()?;
    let mut items = vec![];
    for (i, var) in vars.iter().enumerate() {
        if sol.val(var) > 0.5 {
            items.push(i);
        }
    }
    let value = sol.obj_val();

    assert!(items.iter().map(|i| sizes[*i]).sum::<f64>() <= capacity);

    Some((items, value))
}

struct RyanFoster;

impl BranchRule for RyanFoster {
    fn execute(
        &mut self,
        mut model: Model<Solving>,
        _branchrule: SCIPBranchRule,
        candidates: Vec<BranchingCandidate>,
    ) -> BranchingResult {
        let fractional_pair = RyanFoster::find_fractional_pair(
            &model,
            model.get_data::<PatternForVar>().unwrap(),
            &candidates,
        );
        // println!("-- Branching on fractional pair: {:?}", fractional_pair);

        let current_bb_node = model.focus_node().number();
        let current_decisions = model
            .get_data::<BranchingDecisionMap>()
            .unwrap()
            .0
            .get(&current_bb_node)
            .unwrap()
            .clone();

        // save branching decisions (for the pricer)
        let down_child = model.create_child();
        let up_child = model.create_child();
        let mut down_decisions = current_decisions.clone();
        down_decisions.apart.insert(fractional_pair.clone());
        let mut up_decisions = current_decisions.clone();
        up_decisions.together.insert(fractional_pair.clone());
        model
            .get_data_mut::<BranchingDecisionMap>()
            .unwrap()
            .0
            .insert(down_child.number(), down_decisions);
        model
            .get_data_mut::<BranchingDecisionMap>()
            .unwrap()
            .0
            .insert(up_child.number(), up_decisions);

        // fix infeasible variables
        let (i, j) = (fractional_pair.0, fractional_pair.1);
        for var in model.vars().iter() {
            // skip fixed vars
            if var.ub_local() < model.eps() {
                continue;
            }

            let pattern = model
                .get_data::<PatternForVar>()
                .unwrap()
                .0
                .get(&var.index())
                .unwrap()
                .clone();

            let pattern_set = HashSet::<&usize>::from_iter(pattern.iter());

            // down child: fix any variable that uses both nodes of the pair
            if pattern_set.contains(&i) && pattern_set.contains(&j) {
                model.set_ub_node(&down_child, var, 0.0);
            }

            // up child: fix any variable that uses neither node of the pair
            let neither_is_in_pattern = !pattern_set.contains(&i) && !pattern_set.contains(&j);
            let both_are_in_pattern = pattern_set.contains(&i) && pattern_set.contains(&j);
            if !(both_are_in_pattern || neither_is_in_pattern) {
                model.set_ub_node(&up_child, var, 0.0);
            }
        }

        BranchingResult::CustomBranching
    }
}

impl RyanFoster {
    fn find_fractional_pair(
        model: &Model<Solving>,
        pattern_for_var: &PatternForVar,
        candidates: &Vec<BranchingCandidate>,
    ) -> Pair {
        let mut pair_vals = BTreeMap::new();
        for candidate in candidates {
            let var = model.var_in_prob(candidate.var_prob_id).unwrap();
            let pattern = pattern_for_var.0.get(&var.index()).unwrap();

            for i in 0..pattern.len() - 1 {
                for j in i + 1..pattern.len() {
                    let item_i = pattern[i];
                    let item_j = pattern[j];

                    if item_i != item_j {
                        let pair = (item_i, item_j);
                        let val = pair_vals.entry(pair).or_insert(0.0);
                        *val += candidate.lp_sol_val;
                    }
                }
            }
        }

        // find the pair with the largest fractional value
        let pair = pair_vals
            .iter()
            .filter(|(_, &val)| val.fract() > model.eps() && val < 1.0 - model.eps())
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;

        Pair(pair.0, pair.1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_packing() {
        main();
    }
}
