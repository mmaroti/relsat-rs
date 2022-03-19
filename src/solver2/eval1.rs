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

use std::rc::Rc;

use super::{Clause, Coord, Literal, State, UniversalFormula};

#[derive(Debug, Clone, Copy)]
pub enum EvalStep {
    Loop(u32),
    Atom(u32),
}

#[derive(Debug)]
pub struct Evaluator {
    pub formula: Rc<UniversalFormula>,
    pub program: Box<[EvalStep]>,
}

impl Evaluator {
    pub fn watch(&self, state: &mut State, lit: &Literal) -> Option<Clause> {
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

    pub fn propagate(&self, state: &mut State, coords: &mut [Coord], step: usize) -> bool {
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
