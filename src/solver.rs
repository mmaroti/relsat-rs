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

use super::buffer::Buffer2;
use super::shape::Shape;
use super::theory::Theory;

#[derive(Debug)]
pub struct Relation {
    buffer: Buffer2,
    shape: Shape,
}

impl Relation {
    fn new(size: usize, arity: usize) -> Self {
        let shape = Shape::new(vec![size, arity]);
        let buffer = Buffer2::new(shape.size());
        Self { buffer, shape }
    }

    fn size(&self) -> usize {
        assert!(self.shape.rank() > 0);
        self.shape[0]
    }

    pub fn set_equ(&mut self) {
        assert!(self.shape.rank() == 2);
        let size = self.shape[0];
        for i in 0..size {
            for j in 0..size {
                let pos = self.shape.position(&[i, j]);
                self.buffer.set(pos, if i == j { 2 } else { 1 })
            }
        }
    }
}

impl<'a> fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        const TABLE: [char; 4] = ['?', '0', '1', 'X'];
        for idx in 0..self.buffer.len() {
            let val = self.buffer.get(idx);
            write!(f, "{}", TABLE[val as usize])?;
        }
        write!(f, "\"")
    }
}

#[derive(Debug)]
pub struct Solver {
    theory: Theory,
    size: usize,
    relations: Vec<Relation>,
}

impl Solver {
    pub fn new(theory: Theory, size: usize) -> Self {
        let relations = theory
            .variables
            .iter()
            .map(|var| Relation::new(size, var.arity))
            .collect();
        Self {
            theory,
            size,
            relations,
        }
    }

    pub fn get_relation(&mut self, var: isize) -> &mut Relation {
        assert!(var >= 1);
        &mut self.relations[var as usize - 1]
    }

    pub fn print(&self) {
        for (idx, rel) in self.relations.iter().enumerate() {
            println!("relation: {} {}", self.theory.variables[idx].name, rel);
        }
    }
}
