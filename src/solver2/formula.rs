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

use std::fmt;
use std::rc::Rc;

use super::{get_coords, get_offset, Coord, Domain, Literal, LiteralIdx, Predicate};

#[derive(Debug)]
pub struct AtomicFormula {
    negated: bool,
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
}

impl AtomicFormula {
    pub fn new(negated: bool, predicate: Rc<Predicate>, variables: Vec<usize>) -> Self {
        let variables = variables.into_boxed_slice();
        assert_eq!(predicate.arity(), variables.len());

        Self {
            negated,
            predicate,
            variables,
        }
    }

    pub fn negated(&self) -> bool {
        self.negated
    }

    pub fn predicate(&self) -> &Rc<Predicate> {
        &self.predicate
    }

    pub fn variables(&self) -> &[usize] {
        &self.variables
    }

    pub fn get_literal(&self, coords: &[Coord]) -> LiteralIdx {
        let offset = self
            .predicate
            .get_offset(self.variables.iter().map(|&i| coords[i]));
        LiteralIdx::new(self.negated, offset)
    }
}

impl fmt::Display for AtomicFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}(",
            if self.negated { '-' } else { '+' },
            self.predicate.name()
        )?;
        let mut first = true;
        for &var in self.variables.iter() {
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
pub struct UniversalFormula {
    domains: Box<[Rc<Domain>]>,
    disjunction: Box<[AtomicFormula]>,
    cla_start: usize,
    cla_count: usize,
}

impl UniversalFormula {
    pub fn new<ITER>(disjunction: ITER, cla_start: usize) -> Self
    where
        ITER: ExactSizeIterator<Item = (bool, Rc<Predicate>, Vec<usize>)>,
    {
        let mut domains: Vec<Option<Rc<Domain>>> = Default::default();
        let disjunction: Vec<AtomicFormula> = disjunction
            .map(|(neg, pred, vars)| {
                for (pos, &var) in vars.iter().enumerate() {
                    if domains.len() <= var {
                        domains.resize(var + 1, None);
                    }
                    let dom1 = pred.domain(pos);
                    let dom2 = &mut domains[var];
                    if let Some(dom2) = dom2 {
                        assert!(dom1.ptr_eq(dom2));
                    } else {
                        *dom2 = Some(dom1.clone());
                    }
                }
                AtomicFormula::new(neg, pred, vars)
            })
            .collect();

        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|d| d.unwrap()).collect();
        let cla_count = domains.iter().map(|dom| dom.size()).product();

        Self {
            domains: domains.into_boxed_slice(),
            disjunction: disjunction.into_boxed_slice(),
            cla_start,
            cla_count,
        }
    }

    pub fn arity(&self) -> usize {
        self.domains.len()
    }

    pub fn domain(&self, var: usize) -> &Rc<Domain> {
        &self.domains[var]
    }

    pub fn disjunction(&self, pos: usize) -> &AtomicFormula {
        &self.disjunction[pos]
    }

    pub fn cla_count(&self) -> usize {
        self.cla_count
    }

    pub fn get_coords(&self, offset: usize, coords: &mut [Coord]) {
        get_coords(&self.domains, offset, coords);
    }

    pub fn get_offset<I>(&self, coords: I) -> usize
    where
        I: ExactSizeIterator<Item = Coord>,
    {
        get_offset(&self.domains, coords)
    }
}

impl fmt::Display for UniversalFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for atom in self.disjunction.iter() {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }
            write!(f, "{}", atom)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClauseIdx(pub usize);

#[derive(Debug)]
pub struct Clause<'a> {
    formula: &'a Rc<UniversalFormula>,
    coords: Vec<Coord>,
}

impl<'a> Clause<'a> {
    pub fn new(formula: &'a Rc<UniversalFormula>, coords: Vec<Coord>) -> Self {
        debug_assert_eq!(coords.len(), formula.arity());
        Self { formula, coords }
    }

    pub fn idx(&self) -> ClauseIdx {
        let cla_offset = self.formula.get_offset(self.coords.iter().cloned());
        ClauseIdx(self.formula.cla_start + cla_offset)
    }

    pub fn literals(&self) -> Vec<Literal> {
        self.formula
            .disjunction
            .iter()
            .map(|atom| {
                Literal::new(
                    atom.negated,
                    &atom.predicate,
                    atom.variables.iter().map(|&var| self.coords[var]).collect(),
                )
            })
            .collect()
    }

    pub fn destroy(self) -> Vec<Coord> {
        self.coords
    }
}

impl<'a> fmt::Display for Clause<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut coords = Vec::new();
        let mut first = true;
        for atom in self.formula.disjunction.iter() {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }

            coords.clear();
            coords.extend(atom.variables.iter().map(|&var| self.coords[var]));
            let lit = Literal::new(atom.negated, &atom.predicate, coords);
            write!(f, "{}", lit)?;
            coords = lit.destroy();
        }
        Ok(())
    }
}
