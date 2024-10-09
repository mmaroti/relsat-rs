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

#[derive(Debug, Clone, Copy)]
pub struct Rel(usize);

struct Literal {
    sign: bool,
    relation: Rel,
    variables: Box<[usize]>,
}

#[derive(Debug, Default)]
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
        let mut domains: Vec<Option<Dom>> = Default::default();
        for (_, rel, vars) in literals.iter() {
            let rel = &self.relations[rel.0];
            assert_eq!(rel.domains.len(), vars.len());
            for (pos, &var) in vars.iter().enumerate() {
                if domains.len() <= var {
                    domains.resize(var + 1, None);
                }
                let dom1 = rel.domains[pos];
                let dom2 = &mut domains[var];
                if dom2.is_none() {
                    *dom2 = Some(dom1);
                } else {
                    assert_eq!(dom1, dom2.unwrap());
                }
            }
        }
        let domains: Vec<Dom> = domains.into_iter().map(|d| d.unwrap()).collect();

        let literals: Vec<Literal> = literals
            .into_iter()
            .map(|(sign, rel, vars)| Literal {
                relation: rel,
                sign,
                variables: vars.into_boxed_slice(),
            })
            .collect();

        // let cla = Clause::new(shape, domains, literals);
        // self.clauses.push(cla);
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
            print!("relation {} = [", rel.name);
            for (idx, &dom) in rel.domains.iter().enumerate() {
                if idx != 0 {
                    print!(", ")
                }
                let dom = &self.domains[dom.0];
                print!("{}", dom.name)
            }
            println!("]");
        }
    }
}
