/*
* Copyright (C) 2019-2021, Miklos Maroti
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::fmt;

struct SolverItem<'a, ITEM: ?Sized>(&'a Solver, &'a ITEM);

#[derive(Debug)]
struct Domain {
    name: String,
    size: usize,
}

impl Domain {
    fn new(name: String, size: usize) -> Self {
        Self { name, size }
    }
}

#[derive(Debug, Default, Clone)]
struct Variable {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainIdx(usize);

#[derive(Debug)]
struct Predicate {
    name: String,
    domains: Box<[DomainIdx]>,
    first_variable: usize,
    num_variables: usize,
}

impl Predicate {
    fn new(solver: &Solver, name: String, domains: Vec<DomainIdx>) -> Self {
        let domains = domains.into_boxed_slice();
        let first_variable = solver.num_variables();
        let num_variables = domains
            .iter()
            .map(|&dom| solver.get_domain(dom).size)
            .product();
        Self {
            name,
            domains,
            first_variable,
            num_variables,
        }
    }

    fn arity(&self) -> usize {
        self.domains.len()
    }
}

impl<'a> fmt::Display for SolverItem<'a, Predicate> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.1.name)?;
        let mut first = true;
        for &dom in self.1.domains.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            let dom = self.0.get_domain(dom);
            write!(f, "{}", dom.name)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredicateIdx(usize);

#[derive(Debug)]
struct AtomicFormula {
    sign: bool,
    predicate: PredicateIdx,
    variables: Box<[usize]>,
}

impl AtomicFormula {
    fn new(
        solver: &Solver,
        domains: &mut Vec<Option<DomainIdx>>,
        sign: bool,
        predicate: PredicateIdx,
        variables: Vec<usize>,
    ) -> Self {
        let variables = variables.into_boxed_slice();

        let pred = solver.get_predicate(predicate);
        assert_eq!(pred.domains.len(), variables.len());
        for (pos, &var) in variables.iter().enumerate() {
            if domains.len() <= var {
                domains.resize(var + 1, None);
            }
            let dom1 = pred.domains[pos];
            let dom2 = &mut domains[var];
            if let Some(dom2) = dom2 {
                assert_eq!(dom1, *dom2);
            } else {
                *dom2 = Some(dom1);
            }
        }

        Self {
            sign,
            predicate,
            variables,
        }
    }
}

impl<'a> fmt::Display for SolverItem<'a, AtomicFormula> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pred = self.0.get_predicate(self.1.predicate);
        write!(f, "{}{}(", if self.1.sign { '+' } else { '-' }, pred.name)?;
        let mut first = true;
        for &var in self.1.variables.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "x{}", var)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
struct UniversalFormula {
    domains: Box<[DomainIdx]>,
    disjunction: Box<[AtomicFormula]>,
}

impl UniversalFormula {
    fn new(solver: &Solver, disjunction: Vec<(bool, PredicateIdx, Vec<usize>)>) -> Self {
        let mut domains: Vec<Option<DomainIdx>> = Default::default();
        let disjunction: Vec<AtomicFormula> = disjunction
            .into_iter()
            .map(|(sign, pred, vars)| AtomicFormula::new(solver, &mut domains, sign, pred, vars))
            .collect();

        let domains: Vec<DomainIdx> = domains.into_iter().map(|d| d.unwrap()).collect();
        Self {
            domains: domains.into_boxed_slice(),
            disjunction: disjunction.into_boxed_slice(),
        }
    }
}

impl<'a> fmt::Display for SolverItem<'a, UniversalFormula> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for atom in self.1.disjunction.iter() {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }
            write!(f, "{}", SolverItem(self.0, atom))?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    domains: Vec<Domain>,
    variables: Vec<Variable>,
    predicates: Vec<Predicate>,
    formulas: Vec<UniversalFormula>,
}

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> DomainIdx {
        let idx = DomainIdx(self.domains.len());
        self.domains.push(Domain::new(name, size));
        idx
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    fn get_domain(&self, idx: DomainIdx) -> &Domain {
        &self.domains[idx.0]
    }

    pub fn add_predicate(&mut self, name: String, domains: Vec<DomainIdx>) -> PredicateIdx {
        let idx = PredicateIdx(self.predicates.len());
        let pred = Predicate::new(self, name, domains);
        self.variables.resize(
            self.variables.len() + pred.num_variables,
            Default::default(),
        );
        self.predicates.push(pred);
        idx
    }

    fn get_predicate(&self, idx: PredicateIdx) -> &Predicate {
        &self.predicates[idx.0]
    }

    pub fn add_formula(&mut self, disjunction: Vec<(bool, PredicateIdx, Vec<usize>)>) {
        let formula = UniversalFormula::new(self, disjunction);
        self.formulas.push(formula);
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {} = {}", dom.name, dom.size);
        }
        for pred in self.predicates.iter() {
            println!("predicate {}", SolverItem(self, pred));
        }
        for form in self.formulas.iter() {
            println!("formula {}", SolverItem(self, form));
        }
    }
}
