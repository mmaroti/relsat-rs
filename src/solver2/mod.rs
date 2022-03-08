/*
* Copyright (C) 2019-2022, Miklos Maroti
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

mod domain;
mod formula;
mod predicate;

use domain::{get_coords, get_offset, Coord, Domain};
use formula::{Clause, ClauseIdx, UniversalFormula};
use predicate::{Literal, LiteralIdx, Predicate};

use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
enum EvalStep {
    Loop(u32),
    Atom(u32),
}

#[derive(Debug)]
struct Evaluator {
    formula: Rc<UniversalFormula>,
    program: Box<[EvalStep]>,
}

impl Evaluator {
    fn watch(&self, state: &mut State, lit: &Literal) -> Option<Clause> {
        if let Some(&EvalStep::Atom(atom)) = self.program.first() {
            let atom = self.formula.disjunction(atom as usize);
            debug_assert_eq!(atom.negated(), lit.negated());
            debug_assert!(atom.predicate().ptr_eq(lit.predicate()));
            debug_assert!(state.get_value(lit.idx()) < 0);

            let mut coords = vec![Coord(usize::MAX); self.formula.arity()];
            for (&var, &coord) in atom.variables().iter().zip(lit.coords()) {
                debug_assert_ne!(coord, Coord(usize::MAX));
                if coords[var] != Coord(usize::MAX) {
                    return None;
                }
                coords[var] = coord;
            }
            if self.propagate(state, &mut coords, 1) {
                Some(Clause::new(&self.formula, coords))
            } else {
                None
            }
        } else {
            panic!();
        }
    }

    fn propagate(&self, state: &mut State, coords: &mut [Coord], step: usize) -> bool {
        match self.program.get(step) {
            None => true,
            Some(&EvalStep::Atom(atom)) => {
                let lit = self.formula.disjunction(atom as usize).get_literal(coords);
                let val = state.get_value(lit);
                if val < 0 {
                    self.propagate(state, coords, step + 1)
                } else {
                    if val == 0 && self.conflicting(state, coords, step + 1) {
                        state.enqueue(lit);
                    }
                    false
                }
            }
            Some(&EvalStep::Loop(var)) => {
                let size = self.formula.domain(var as usize).size();
                debug_assert_eq!(coords[var as usize], Coord(usize::MAX));
                for coord in 0..size {
                    coords[var as usize] = Coord(coord);
                    if self.propagate(state, coords, step + 1) {
                        coords[var as usize] = Coord(usize::MAX);
                        return true;
                    }
                }
                coords[var as usize] = Coord(usize::MAX);
                false
            }
        }
    }

    fn conflicting(&self, state: &State, coords: &mut [Coord], step: usize) -> bool {
        match self.program.get(step) {
            None => true,
            Some(&EvalStep::Atom(idx)) => {
                let atom = self.formula.disjunction(idx as usize);
                if state.get_value(atom.get_literal(coords)) < 0 {
                    self.conflicting(state, coords, step + 1)
                } else {
                    false
                }
            }
            Some(&EvalStep::Loop(var)) => {
                let size = self.formula.domain(var as usize).size();
                debug_assert_eq!(coords[var as usize], Coord(usize::MAX));
                for coord in 0..size {
                    coords[var as usize] = Coord(coord);
                    if self.conflicting(state, coords, step + 1) {
                        coords[var as usize] = Coord(usize::MAX);
                        return true;
                    }
                }
                coords[var as usize] = Coord(usize::MAX);
                false
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct State {
    values: Vec<i8>,
}

impl State {
    fn get_variables(&self) -> usize {
        self.values.len()
    }

    fn set_variables(&mut self, count: usize) {
        self.values.resize(count, 0);
    }

    fn get_value(&self, lit: LiteralIdx) -> i8 {
        let val = self.values[lit.variable()];
        if lit.negated() {
            -val
        } else {
            val
        }
    }

    /// Sets the given literal to true and enqueues it for unit propagation.
    fn enqueue(&mut self, lit: LiteralIdx) {
        let var = lit.variable();
        assert_eq!(self.values[var], 0);
        self.values[var] = if lit.negated() { -1 } else { 1 };
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: State,
    domains: Vec<Rc<Domain>>,
    predicates: Vec<Rc<Predicate>>,
    formulas: Vec<Rc<UniversalFormula>>,
    cla_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DomainIdx(usize);

#[derive(Debug, Clone, Copy)]
pub struct PredicateIdx(usize);

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> DomainIdx {
        let idx = DomainIdx(self.domains.len());
        self.domains.push(Rc::new(Domain::new(name, size)));
        idx
    }

    pub fn add_predicate(&mut self, name: String, domains: Vec<DomainIdx>) -> PredicateIdx {
        let domains: Vec<Rc<Domain>> = domains
            .into_iter()
            .map(|idx| self.domains[idx.0].clone())
            .collect();
        let idx = PredicateIdx(self.predicates.len());
        let pred = Rc::new(Predicate::new(name, domains, self.state.get_variables()));
        self.state
            .set_variables(self.state.get_variables() + pred.var_count());
        self.predicates.push(pred);
        idx
    }

    pub fn add_formula(&mut self, disjunction: Vec<(bool, PredicateIdx, Vec<usize>)>) {
        let disjunction = disjunction
            .into_iter()
            .map(|(neg, pred, vars)| (neg, self.predicates[pred.0].clone(), vars));
        let formula = Rc::new(UniversalFormula::new(disjunction, self.cla_count));
        self.cla_count += formula.cla_count();
        self.formulas.push(formula);
    }

    fn get_literal(&self, idx: LiteralIdx) -> Literal {
        let negated = idx.negated();
        let mut offset = idx.variable();
        for predicate in self.predicates.iter() {
            if offset < predicate.var_count() {
                let mut coords = vec![Coord(0); predicate.arity()];
                predicate.get_coords(offset, &mut coords);
                let lit = Literal::new(negated, predicate, coords);
                debug_assert_eq!(lit.idx(), idx);
                return lit;
            }
            offset -= predicate.var_count();
        }
        panic!();
    }

    fn get_clause(&self, idx: ClauseIdx) -> Clause {
        let mut offset = idx.0;
        for formula in self.formulas.iter() {
            if offset < formula.cla_count() {
                let mut coords = vec![Coord(0); formula.arity()];
                formula.get_coords(offset, &mut coords);
                let cla = Clause::new(formula, coords);
                debug_assert_eq!(cla.idx(), idx);
                return cla;
            }
            offset -= formula.cla_count();
        }
        panic!();
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {} = {}", dom, dom.size());
        }
        for pred in self.predicates.iter() {
            println!("predicate {}", pred);
        }
        for form in self.formulas.iter() {
            println!("formula {}", form);
        }
        println!("variable count {}", self.state.get_variables());
        println!("clause count {}", self.cla_count);
    }

    pub fn test(&mut self) {
        let watcher = Evaluator {
            formula: self.formulas[1].clone(),
            program: vec![EvalStep::Atom(0), EvalStep::Atom(1)].into(),
        };

        let lit1 = Literal::new(true, &self.predicates[0], vec![Coord(1), Coord(2)]);
        self.state.enqueue(!lit1.idx());
        let lit2 = Literal::new(false, &self.predicates[0], vec![Coord(2), Coord(1)]);
        // self.state.enqueue(!lit2.idx());

        println!("{:?}", watcher.watch(&mut self.state, &lit1));
        println!("{}", self.state.get_value(lit1.idx()));
        println!("{}", self.state.get_value(lit2.idx()));
    }
}
