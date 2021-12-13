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

#![allow(dead_code)]

mod bitops;
mod buffer;
mod shape;
mod solver;
mod theory;
mod tokenizer;

use solver::*;

fn main() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set", 2);
    let equ = sol.add_variable("equ", vec![&set, &set]);
    let mul = sol.add_variable("mul", vec![&set, &set, &set]);
    let inv = sol.add_variable("inv", vec![&set, &set]);
    let one = sol.add_variable("one", vec![&set]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 3]),
        (false, &mul, vec![3, 2, 4]),
        (false, &mul, vec![1, 2, 5]),
        (true, &mul, vec![0, 5, 4]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 3]),
        (false, &mul, vec![0, 3, 4]),
        (false, &mul, vec![0, 1, 5]),
        (true, &mul, vec![5, 2, 4]),
    ]);

    sol.add_clause(vec![
        (false, &inv, vec![0, 1]),
        (false, &mul, vec![1, 0, 2]),
        (true, &one, vec![2]),
    ]);

    sol.add_clause(vec![(false, &one, vec![0]), (true, &mul, vec![0, 1, 1])]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 2]),
        (false, &mul, vec![0, 1, 3]),
        (true, &equ, vec![2, 3]),
    ]);

    sol.add_clause(vec![
        (false, &inv, vec![0, 1]),
        (false, &inv, vec![0, 2]),
        (true, &equ, vec![1, 2]),
    ]);

    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &one, vec![1]),
        (true, &equ, vec![0, 1]),
    ]);

    if false {
        // learnt
        sol.add_clause(vec![
            (true, &equ, vec![0, 1]),
            (false, &one, vec![0]),
            (false, &mul, vec![1, 1, 0]),
            (true, &mul, vec![1, 0, 1]),
        ]);

        // learnt
        sol.add_clause(vec![
            (true, &equ, vec![0, 1]),
            (false, &mul, vec![0, 1, 0]),
            (false, &mul, vec![1, 1, 0]),
            (true, &mul, vec![1, 0, 0]),
        ]);
    }

    sol.set_equality(&equ);
    sol.search_all();

    // sol.set_value(&mul, &[0, 0, 0], true);
    // sol.set_value(&mul, &[1, 0, 1], false);
    // sol.set_value(&mul, &[0, 1, 0], true);
    // sol.set_value(&mul, &[1, 0, 0], false);
    // sol.set_value(&mul, &[1, 1, 0], true);

    // sol.propagate();
    // sol.print();
}
