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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Literal {
    variable: isize,
    indices: Box<[usize]>,
}

impl Literal {
    pub fn new(variable: isize, indices: Vec<usize>) -> Self {
        debug_assert!(variable != 0);
        let indices = indices.into_boxed_slice();
        Self { variable, indices }
    }

    pub fn sign(&self) -> bool {
        self.variable >= 0
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

#[derive(Clone, Debug, Default)]
pub struct Theory {
    pub(crate) variables: Vec<Variable>,
    clauses: Vec<Clause>,
}

impl Theory {
    pub fn add_variable(&mut self, name: &str, arity: usize) -> isize {
        assert!(!self.variables.iter().any(|v| v.name == name));
        let var = (self.variables.len() + 1) as isize;
        self.variables.push(Variable::new(name, arity));
        var
    }

    pub fn get_variable(&self, var: isize) -> &Variable {
        debug_assert!(var != 0);
        let var = (var.abs() - 1) as usize;
        &self.variables[var]
    }

    pub fn add_clause(&mut self, arity: usize, literals: Vec<(isize, Vec<usize>)>) {
        let mut used = vec![false; arity];
        let literals = literals
            .into_iter()
            .map(|(var, map)| {
                assert!(self.get_variable(var).arity == map.len());
                for &x in map.iter() {
                    assert!(x < arity);
                    used[x] = true;
                }
                Literal::new(var, map)
            })
            .collect();
        assert!(used.iter().all(|&x| x));
        self.clauses.push(Clause::new(literals));
    }

    pub fn print(&self) {
        for var in self.variables.iter() {
            println!("variable: {}", self.context(var));
        }
        for cla in self.clauses.iter() {
            println!("clause: {}", self.context(cla));
        }
    }

    fn context<'a, OBJ>(&'a self, obj: &'a OBJ) -> Context<'a, OBJ> {
        Context { obj, thy: self }
    }
}

struct Context<'a, OBJ> {
    obj: &'a OBJ,
    thy: &'a Theory,
}

impl<'a> fmt::Display for Context<'a, Variable> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.obj.name)?;
        for idx in 0..self.obj.arity {
            if idx > 0 {
                write!(f, ",")?;
            }
            write!(f, "x{}", idx)?;
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for Context<'a, Literal> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.thy.get_variable(self.obj.variable);
        write!(f, "{}{}(", if self.obj.sign() { '+' } else { '-' }, v.name)?;
        let mut first = true;
        for &idx in self.obj.indices.iter() {
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

impl<'a> fmt::Display for Context<'a, Clause> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for lit in self.obj.literals.iter() {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", self.thy.context(lit))?;
        }
        Ok(())
    }
}
