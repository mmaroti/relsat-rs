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

use crate::solver1::bitops::*;
use crate::solver1::buffer::Buffer2;
use crate::solver1::shape::Shape;

#[derive(Debug)]
struct Domain {
    size: usize,
    name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Dom(usize);

#[derive(Debug)]
struct Variable {
    shape: Shape,
    name: String,
    domains: Box<[Dom]>,
}

#[derive(Debug, Clone, Copy)]
pub struct Var(usize);

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
    variables: Vec<Variable>,
}

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> Dom {
        assert!(self.domains.iter().all(|dom| dom.name != name));
        let dom = self.domains.len();
        self.domains.push(Domain { name, size });
        Dom(dom)
    }

    pub fn add_variable(&mut self, name: String, domains: Vec<Dom>) -> Var {
        assert!(self.variables.iter().all(|var| var.name != name));

        let var = self.variables.len();

        let shape = Shape::new(
            domains.iter().map(|dom| self.domains[dom.0].size).collect(),
            self.assignment.len(),
        );
        self.assignment.append(shape.volume(), BOOL_UNDEF);

        let domains = domains.into_boxed_slice();
        self.variables.push(Variable {
            shape,
            name,
            domains,
        });

        Var(var)
    }

    fn assign(&mut self, pos: usize, sign: bool, reason: Vec<usize>) {
        assert!(self.assignment.get(pos) == BOOL_UNDEF);
        self.assignment
            .set(pos, if sign { BOOL_TRUE } else { BOOL_FALSE });
        self.steps.push(Step { pos, reason });
    }

    pub fn set_value(&mut self, sign: bool, var: Var, coordinates: &[usize]) {
        let var = &self.variables[var.0];
        let pos = var.shape.position(coordinates.iter());
        self.assign(pos, sign, vec![]);
    }
}
