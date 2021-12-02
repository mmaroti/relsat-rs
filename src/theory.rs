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

#[derive(Debug)]
pub struct Variable {
    pub arity: usize,
    pub name: String,
}

impl Variable {
    pub fn new(name: &str, arity: usize) -> Self {
        let name = name.to_string();
        Self { arity, name }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for idx in 0..self.arity {
            if idx > 0 {
                write!(f, ",")?;
            }
            write!(f, "x{}", idx)?;
        }
        write!(f, ")")
    }
}

#[derive(Clone, Debug)]
pub struct Literal {
    variable: Rc<Variable>,
    sign: bool,
    indices: Box<[usize]>,
}

impl Literal {
    pub fn new(variable: Rc<Variable>, sign: bool, indices: Vec<usize>) -> Self {
        assert!(variable.arity == indices.len());
        let indices = indices.into_boxed_slice();
        Self {
            variable,
            indices,
            sign,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}(",
            if self.sign { '+' } else { '-' },
            self.variable.name
        )?;
        let mut first = true;
        for &idx in self.indices.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "x{}", idx)?;
        }
        write!(f, ")")
    }
}

#[derive(Clone, Debug)]
pub struct Clause {
    literals: Box<[Literal]>,
}

impl Clause {
    fn new(literals: Vec<Literal>) -> Self {
        let literals = literals.into_boxed_slice();
        Self { literals }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for lit in self.literals.iter() {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", lit)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Theory {
    pub(crate) variables: Vec<Rc<Variable>>,
    clauses: Vec<Clause>,
}

impl Theory {
    pub fn add_variable(&mut self, name: &str, arity: usize) -> Rc<Variable> {
        assert!(self.variables.iter().all(|var| var.name != name));
        let var = Rc::new(Variable::new(name, arity));
        self.variables.push(var.clone());
        var
    }

    pub fn add_clause(&mut self, arity: usize, literals: Vec<(bool, &Rc<Variable>, Vec<usize>)>) {
        let mut used = vec![false; arity];
        let literals = literals
            .into_iter()
            .map(|(sgn, var, map)| {
                for &x in map.iter() {
                    assert!(x < arity);
                    used[x] = true;
                }
                Literal::new(var.clone(), sgn, map)
            })
            .collect();
        assert!(used.iter().all(|&x| x));
        self.clauses.push(Clause::new(literals));
    }

    pub fn print(&self) {
        for var in self.variables.iter() {
            println!("variable: {}", var);
        }
        for cla in self.clauses.iter() {
            println!("clause: {}", cla);
        }
    }
}
