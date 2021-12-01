/*
* Copyright (C) 2019-2020, Miklos Maroti
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
    variable: usize,
    sign: bool,
    vars: Vec<usize>,
    arity: usize,
}

impl Literal {
    pub fn new(variable: usize, sign: bool, vars: Vec<usize>, arity: usize) -> Self {
        for &var in vars.iter() {
            assert!(var < arity);
        }
        Self {
            variable,
            sign,
            vars,
            arity,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Clause {
    literals: Vec<Literal>,
}

impl Clause {
    fn new(literals: Vec<Literal>) -> Self {
        Self { literals }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Theory {
    variables: Vec<Variable>,
    clauses: Vec<Clause>,
}

impl Theory {
    pub fn add_variable(&mut self, name: &str, arity: usize) -> usize {
        assert!(!self.variables.iter().any(|v| v.name == name));
        let v = self.variables.len();
        self.variables.push(Variable::new(name, arity));
        v
    }

    pub fn add_clause(&mut self, arity: usize, literals: Vec<(usize, bool, Vec<usize>)>) {
        let mut used = vec![false; arity];
        let literals = literals
            .into_iter()
            .map(|(v, s, m)| {
                for &x in m.iter() {
                    used[x] = true;
                }
                assert!(self.variables[v].arity == m.len());
                Literal::new(v, s, m, arity)
            })
            .collect();
        assert!(used.iter().all(|&x| x));
        self.clauses.push(Clause::new(literals));
    }

    pub fn print(&self) {
        for v in self.variables.iter() {
            println!("variable: {}", self.context(v));
        }
        for c in self.clauses.iter() {
            println!("clause: {}", self.context(c));
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
        for i in 0..self.obj.arity {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "x{}", i)?;
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for Context<'a, Literal> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = &self.thy.variables[self.obj.variable];
        write!(f, "{}{}(", if self.obj.sign { '+' } else { '-' }, v.name)?;
        for (i, x) in self.obj.vars.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "x{}", x)?;
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for Context<'a, Clause> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, l) in self.obj.literals.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", self.thy.context(l))?;
        }
        Ok(())
    }
}
