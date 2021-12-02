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
use std::rc::Rc;

use super::buffer::Buffer2;
use super::shape::Shape;
use super::theory::{Theory, Variable};

#[derive(Debug)]
pub struct Relation {
    variable: Rc<Variable>,
    buffer: Buffer2,
    shape: Shape,
}

impl Relation {
    fn new(variable: Rc<Variable>, size: usize) -> Self {
        let shape = Shape::new(vec![size; variable.arity]);
        let buffer = Buffer2::new(shape.size());
        Self {
            variable,
            buffer,
            shape,
        }
    }

    fn size(&self) -> usize {
        if self.shape.rank() > 0 {
            self.shape[0]
        } else {
            1
        }
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
        let size = self.size();
        write!(f, "{} \"", self.variable)?;
        const TABLE: [char; 4] = ['?', '0', '1', 'X'];
        for idx in 0..self.buffer.len() {
            if idx > 0 && idx % size == 0 {
                write!(f, " ")?;
            }
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
            .map(|var| Relation::new(var.clone(), size))
            .collect();
        Self {
            theory,
            size,
            relations,
        }
    }

    pub fn get_relation(&mut self, var: &Variable) -> Option<&mut Relation> {
        for rel in self.relations.iter_mut() {
            if std::ptr::eq(&*rel.variable, var) {
                return Some(rel);
            }
        }
        None
    }

    pub fn print(&self) {
        for rel in self.relations.iter() {
            println!("relation: {}", rel);
        }
    }
}
