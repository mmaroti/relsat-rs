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

use std::io::{Result, Write};

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
                Literal::new(v, s, m, arity)
            })
            .collect();
        assert!(used.iter().all(|&x| x));
        self.clauses.push(Clause::new(literals));
    }

    fn write_arguments<Iter>(&self, out: &mut impl Write, iter: Iter) -> Result<()>
    where
        Iter: Iterator<Item = usize>,
    {
        write!(out, "(")?;
        let mut first = true;
        for x in iter {
            if first {
                first = false;
            } else {
                write!(out, ",")?;
            }
            write!(out, "x{}", x)?;
        }
        write!(out, ")")
    }

    fn write_variable(&self, out: &mut impl Write, var: &Variable) -> Result<()> {
        write!(out, "{}", var.name)?;
        self.write_arguments(out, 0..var.arity)
    }

    fn write_literal(&self, out: &mut impl Write, lit: &Literal) -> Result<()> {
        write!(out, "{}", if lit.sign { '+' } else { '-' })?;
        write!(out, "{}", self.variables[lit.variable].name)?;
        self.write_arguments(out, lit.vars.iter().cloned())
    }

    fn write_clause(&self, out: &mut impl Write, cla: &Clause) -> Result<()> {
        let mut first = true;
        for lit in cla.literals.iter() {
            if first {
                first = false;
            } else {
                write!(out, " ")?;
            }
            self.write_literal(out, lit)?;
        }
        Ok(())
    }

    pub fn print(&self) {
        let out = std::io::stdout();
        let mut out = out.lock();
        let steps = || -> std::io::Result<()> {
            for v in self.variables.iter() {
                write!(out, "variable: ")?;
                self.write_variable(&mut out, v)?;
                writeln!(out)?;
            }
            for c in self.clauses.iter() {
                write!(out, "clause: ")?;
                self.write_clause(&mut out, c)?;
                writeln!(out)?;
            }
            Ok(())
        }();
        steps.expect("Could not write to stdout");
    }
}
