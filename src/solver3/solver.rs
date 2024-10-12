/*
* Copyright (C) 2019-2024, Miklos Maroti
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

use super::bitops::*;
use super::buffer::Buffer2;
use super::shape::Shape;

struct State {}

#[derive(Debug)]
struct Domain {
    name: String,
    size: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dom(usize);

#[derive(Debug)]
struct Relation {
    name: String,
    shape: Shape,
    domains: Box<[Dom]>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Rel(usize);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Var(pub usize);

#[derive(Debug)]
struct Literal {
    sign: bool,
    relation: Rel,
    variables: Box<[Var]>,
}

#[derive(Debug)]
struct Clause {
    literals: Box<[Literal]>,
    domains: Box<[Dom]>,
}

#[derive(Debug)]
struct Step {
    pos: usize,
    reason: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct Solver {
    assignment: Buffer2,
    steps: Vec<Step>,
    domains: Vec<Domain>,
    relations: Vec<Relation>,
    clauses: Vec<Clause>,
}

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> Dom {
        assert!(self.domains.iter().all(|dom| dom.name != name));
        let dom = self.domains.len();
        self.domains.push(Domain { name, size });
        Dom(dom)
    }

    pub fn add_relation(&mut self, name: String, domains: Vec<Dom>) -> Rel {
        assert!(self.relations.iter().all(|var| var.name != name));

        let rel = self.relations.len();

        let shape = Shape::new(
            domains.iter().map(|dom| self.domains[dom.0].size),
            self.assignment.len(),
        );
        self.assignment.append(shape.volume(), BOOL_UNDEF);

        let domains = domains.into_boxed_slice();
        self.relations.push(Relation {
            name,
            shape,
            domains,
        });

        Rel(rel)
    }

    pub fn add_clause(&mut self, literals: Vec<(bool, Rel, Vec<usize>)>) {
        let mut domains: Vec<Dom> = Default::default();
        for (_, rel, vars) in literals.iter() {
            let rel = &self.relations[rel.0];
            assert_eq!(rel.domains.len(), vars.len());
            for (pos, &var) in vars.iter().enumerate() {
                if domains.len() <= var {
                    domains.resize(var + 1, Dom(usize::MAX));
                }
                let dom1 = rel.domains[pos];
                let dom2 = &mut domains[var];
                debug_assert!(*dom2 == dom1 || *dom2 == Dom(usize::MAX));
                *dom2 = dom1;
            }
        }

        let literals: Vec<Literal> = literals
            .into_iter()
            .map(|(sign, rel, vars)| {
                let vars: Vec<Var> = vars.into_iter().map(Var).collect();
                Literal {
                    relation: rel,
                    sign,
                    variables: vars.into_boxed_slice(),
                }
            })
            .collect();

        let clause = Clause {
            domains: domains.into_boxed_slice(),
            literals: literals.into_boxed_slice(),
        };
        self.clauses.push(clause);
    }

    fn assign(&mut self, pos: usize, sign: bool, reason: Vec<usize>) {
        assert!(self.assignment.get(pos) == BOOL_UNDEF);
        self.assignment
            .set(pos, if sign { BOOL_TRUE } else { BOOL_FALSE });
        self.steps.push(Step { pos, reason });
    }

    pub fn set_value(&mut self, sign: bool, rel: Rel, coordinates: &[usize]) {
        let shape = &self.relations[rel.0].shape;
        let pos = shape.position(coordinates.iter().cloned());
        self.assign(pos, sign, vec![]);
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {} = {}", dom.name, dom.size);
        }
        for rel in self.relations.iter() {
            println!("relation {} = {}", rel.name, Member(self, &*rel.domains));
        }
        for cla in self.clauses.iter() {
            println!("clause {}", Member(self, cla));
        }
    }
}

struct Member<'a, T>(&'a Solver, T);

impl std::fmt::Display for Member<'_, &[Dom]> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for &dom in self.1.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            let dom = &self.0.domains[dom.0];
            write!(f, "{}", dom.name)?;
        }
        write!(f, "]")
    }
}

impl std::fmt::Display for Member<'_, &Literal> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", if self.1.sign { '+' } else { '-' })?;
        let rel = &self.0.relations[self.1.relation.0];
        write!(f, "{}(", rel.name)?;
        let mut first = true;
        for &var in self.1.variables.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "x{}", var.0)?;
        }
        write!(f, ")")
    }
}

impl std::fmt::Display for Member<'_, &Clause> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        for lit in self.1.literals.iter() {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", Member(self.0, lit))?;
        }
        Ok(())
    }
}
