use std::collections::HashMap;
use std::ops::{Add, Mul, Neg, Sub};
use std::rc::Rc;
use crate::{Variable};

pub struct Expr {
    coeffs: HashMap<Rc<Variable>, f64>,
    constant: f64
}

impl Into<Expr> for f64 {
    fn into(self) -> Expr {
        Expr {
            coeffs: HashMap::new(),
            constant: self
        }
    }
}

impl Into<Expr> for Rc<Variable> {
    fn into(self) -> Expr {
        let mut coeffs = HashMap::new();
        coeffs.insert(self, 1.0);
        Expr {
            coeffs,
            constant: 0.0
        }
    }
}


impl Add for Expr {
    type Output = Expr;

    fn add(self, other: Expr) -> Expr {
        let mut coeffs = self.coeffs;
        for (var, coeff) in other.coeffs {
            coeffs.entry(var).and_modify(|c| *c += coeff).or_insert(coeff);
        }
        Expr {
            coeffs,
            constant: self.constant + other.constant
        }
    }
}


impl Neg for Expr {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        let mut coeffs = self.coeffs;
        for (_, coeff) in coeffs.iter_mut() {
            *coeff = -*coeff;
        }
        Expr {
            coeffs,
            constant: -self.constant
        }
    }
}

impl Sub for Expr {
    type Output = Expr;

    fn sub(self, other: Expr) -> Self::Output {
        self + (-other)
    }
}

impl Mul for Expr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if rhs.coeffs.len() == 0 {
            Expr {
                coeffs: self.coeffs.into_iter().map(|(var, coeff)| (var, coeff * rhs.constant)).collect(),
                constant: self.constant * rhs.constant
            }
        } else if self.coeffs.len() == 0 {
            Expr {
                coeffs: rhs.coeffs.into_iter().map(|(var, coeff)| (var, coeff * self.constant)).collect(),
                constant: self.constant * rhs.constant
            }
        } else {
            panic!("Multiplication of two expressions with multiple variables is not supported");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::Expr;
    use crate::{Model, VarType};

    #[test]
    fn from_constant() {
        let expr: Expr = 3.0.into();
        assert_eq!(expr.coeffs.len(), 0);
        assert_eq!(expr.constant, 3.0);
    }

    #[test]
    fn from_var() {
        let var = Model::default().add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let expr: Expr = var.clone().into();
        assert_eq!(expr.coeffs.len(), 1);
        assert_eq!(expr.coeffs[&var], 1.0);
        assert_eq!(expr.constant, 0.0);
    }

    #[test]
    fn add() {
        let mut model = Model::default();
        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);

        let expr1: Expr = x1.clone().into();
        let expr2: Expr = x2.clone().into();
        let expr3 = expr1 + expr2;
        assert_eq!(expr3.coeffs.len(), 2);
        assert_eq!(expr3.coeffs[&x1], 1.0);
        assert_eq!(expr3.coeffs[&x2], 1.0);
        assert_eq!(expr3.constant, 0.0);
    }

    #[test]
    fn neg() {
        let var = Model::default().add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let expr1: Expr = var.clone().into();
        let expr2 = -expr1;
        assert_eq!(expr2.coeffs.len(), 1);
        assert_eq!(expr2.coeffs[&var], -1.0);
        assert_eq!(expr2.constant, 0.0);
    }

    #[test]
    fn sub() {
        let mut model = Model::default();
        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);

        let expr1: Expr = x1.clone().into();
        let expr2: Expr = x2.clone().into();
        let expr3 = expr1 - expr2;
        assert_eq!(expr3.coeffs.len(), 2);
        assert_eq!(expr3.coeffs[&x1], 1.0);
        assert_eq!(expr3.coeffs[&x2], -1.0);
        assert_eq!(expr3.constant, 0.0);
    }

    #[test]
    fn mul() {
        let var = Model::default().add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let mut expr1: Expr = var.clone().into();
        expr1 = expr1 + 4.0.into();
        let expr2: Expr = 4.0.into();
        let expr3 = expr1 * expr2;
        assert_eq!(expr3.coeffs.len(), 1);
        assert_eq!(expr3.coeffs[&var], 4.0);
        assert_eq!(expr3.constant, 16.0);
    }


    #[test]
    #[should_panic]
    fn mul_panic() {
        let mut model = Model::default();
        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);

        let expr1: Expr = x1.into();
        let expr2: Expr = x2.into();
        let _expr3 = expr1 * expr2;
    }
}
