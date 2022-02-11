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
    let set = sol.add_domain("set", 4);
    let equ = sol.add_variable("equ", vec![&set, &set]);
    let one = sol.add_variable("one", vec![&set]);
    let inv = sol.add_variable("inv", vec![&set, &set]);
    let mul = sol.add_variable("mul", vec![&set, &set, &set]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 3]),
        (false, &mul, vec![3, 2, 4]),
        (false, &mul, vec![1, 2, 5]),
        (true, &mul, vec![0, 5, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 0]),
        (false, &mul, vec![1, 1, 2]),
        (true, &mul, vec![0, 2, 0]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 1]),
        (false, &mul, vec![1, 0, 2]),
        (true, &mul, vec![0, 1, 2]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![1, 2, 3]),
        (true, &mul, vec![0, 3, 3]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 3]),
        (false, &mul, vec![0, 3, 4]),
        (false, &mul, vec![0, 1, 5]),
        (true, &mul, vec![5, 2, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![0, 0, 2]),
        (true, &mul, vec![2, 1, 1]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 1]),
        (false, &mul, vec![0, 1, 2]),
        (true, &mul, vec![1, 0, 2]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 1]),
        (false, &mul, vec![0, 1, 3]),
        (true, &mul, vec![3, 2, 3]),
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

    sol.add_exist(&mul);

    sol.add_clause(vec![
        (false, &inv, vec![0, 1]),
        (false, &inv, vec![0, 2]),
        (true, &equ, vec![1, 2]),
    ]);

    sol.add_exist(&inv);

    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &one, vec![1]),
        (true, &equ, vec![0, 1]),
    ]);

    sol.add_exist(&one);

    sol.set_equality(&equ);
    sol.set_value(&mul, &[0, 0, 0], true);
    sol.set_value(&one, &[1], true);

    if true {
        sol.search_all();
    } else {
        sol.propagate();
        sol.evaluate_all();
        sol.print();
        sol.print_steps();
    }
}

fn main_old() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set", 5);
    let equ = sol.add_variable("equ", vec![&set, &set]);
    let one = sol.add_variable("one", vec![&set]);
    let inv = sol.add_variable("inv", vec![&set, &set]);
    let mul = sol.add_variable("mul", vec![&set, &set, &set]);

    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 3]),
        (false, &mul, vec![3, 2, 4]),
        (false, &mul, vec![1, 2, 5]),
        (true, &mul, vec![0, 5, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 0]),
        (false, &mul, vec![1, 1, 2]),
        (true, &mul, vec![0, 2, 0]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 1]),
        (false, &mul, vec![1, 0, 2]),
        (true, &mul, vec![0, 1, 2]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![1, 2, 3]),
        (true, &mul, vec![0, 3, 3]),
    ]);

    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 3]),
        (false, &mul, vec![0, 3, 4]),
        (false, &mul, vec![0, 1, 5]),
        (true, &mul, vec![5, 2, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, &mul, vec![0, 1, 1]),
        (false, &mul, vec![0, 0, 2]),
        (true, &mul, vec![2, 1, 1]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![0, 0, 1]),
        (false, &mul, vec![0, 1, 2]),
        (true, &mul, vec![1, 0, 2]),
    ]);
    sol.add_clause(vec![
        (false, &mul, vec![1, 2, 1]),
        (false, &mul, vec![0, 1, 3]),
        (true, &mul, vec![3, 2, 3]),
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

    sol.add_exist(&mul);

    sol.add_clause(vec![
        (false, &inv, vec![0, 1]),
        (false, &inv, vec![0, 2]),
        (true, &equ, vec![1, 2]),
    ]);

    sol.add_exist(&inv);

    sol.add_clause(vec![
        (false, &one, vec![0]),
        (false, &one, vec![1]),
        (true, &equ, vec![0, 1]),
    ]);

    sol.add_exist(&one);

    // learnt
    if true {
        sol.add_clause(vec![
            (false, &inv, vec![1, 0]),
            (false, &one, vec![0]),
            (true, &equ, vec![0, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![0, 0]),
            (false, &mul, vec![0, 0, 1]),
            (true, &mul, vec![0, 1, 0]),
        ]);
    }

    if true {
        sol.add_clause(vec![
            (false, &inv, vec![2, 2]),
            (false, &mul, vec![0, 1, 0]),
            (false, &mul, vec![0, 2, 0]),
            (false, &mul, vec![1, 0, 2]),
            (true, &equ, vec![1, 2]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![0, 1]),
            (false, &mul, vec![0, 0, 0]),
            (false, &mul, vec![1, 0, 2]),
            (true, &equ, vec![0, 2]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![2, 2]),
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 2, 0]),
            (true, &equ, vec![0, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![1, 0]),
            (false, &mul, vec![0, 1, 1]),
            (true, &mul, vec![0, 0, 0]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![1, 2]),
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 1, 1]),
            (true, &mul, vec![0, 0, 0]),
        ]);
    }

    if true {
        sol.add_clause(vec![
            (false, &mul, vec![0, 1, 0]),
            (false, &mul, vec![1, 1, 2]),
            (false, &mul, vec![1, 2, 0]),
            (true, &mul, vec![1, 0, 0]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![1, 0]),
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 1, 2]),
            (true, &equ, vec![0, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![2, 0]),
            (false, &mul, vec![0, 2, 3]),
            (false, &mul, vec![2, 1, 2]),
            (true, &mul, vec![0, 2, 1]),
        ]);

        sol.add_clause(vec![
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 2, 1]),
            (true, &mul, vec![1, 2, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![0, 1]),
            (false, &mul, vec![0, 0, 1]),
            // (false, &mul, vec![0, 1, 2]),
            (true, &mul, vec![1, 1, 0]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![2, 3]),
            (false, &mul, vec![0, 0, 3]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 2, 1]),
            (true, &mul, vec![1, 0, 3]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![0, 1]),
            (false, &inv, vec![2, 1]),
            (false, &mul, vec![0, 2, 2]),
            (false, &mul, vec![1, 0, 0]),
            (true, &equ, vec![0, 2]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![2, 2]),
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 1, 3]),
            (false, &mul, vec![0, 2, 1]),
            (true, &mul, vec![1, 3, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![1, 3]),
            (false, &mul, vec![0, 0, 3]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 2, 2]),
            (true, &mul, vec![0, 0, 0]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![2, 0]),
            (false, &mul, vec![0, 0, 3]),
            (false, &mul, vec![0, 1, 3]),
            (false, &mul, vec![0, 2, 1]),
            (true, &mul, vec![0, 1, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![0, 2]),
            (false, &inv, vec![2, 0]),
            (false, &mul, vec![0, 0, 3]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 2, 1]),
            (true, &equ, vec![1, 3]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![1, 1]),
            (false, &mul, vec![0, 0, 1]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![1, 0, 2]),
            // (false, &mul, vec![0, 2, 3]),
            (true, &mul, vec![2, 2, 1]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![2, 2]),
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 2, 1]),
            (true, &mul, vec![1, 1, 2]),
        ]);

        sol.add_clause(vec![
            (false, &inv, vec![0, 3]),
            (false, &mul, vec![3, 3, 0]),
            (false, &mul, vec![3, 2, 3]),
            (false, &mul, vec![0, 3, 1]),
            (true, &equ, vec![1, 2]),
        ]);

        sol.add_clause(vec![
            (false, &mul, vec![3, 1, 1]),
            (false, &mul, vec![0, 0, 2]),
            (false, &mul, vec![0, 1, 2]),
            (false, &mul, vec![0, 2, 3]),
            (true, &mul, vec![3, 0, 1]),
        ]);
    }

    if true {
        sol.add_clause(vec![
            (false, &mul, vec![3, 1, 1]),
            (false, &mul, vec![2, 2, 0]),
            (false, &mul, vec![0, 1, 3]),
            (false, &mul, vec![0, 2, 3]),
            (true, &mul, vec![0, 0, 1]),
        ]);

        sol.add_clause(vec![
            (false, &mul, vec![4, 2, 2]),
            (false, &mul, vec![0, 0, 3]),
            (false, &mul, vec![0, 1, 4]),
            (false, &mul, vec![0, 2, 3]),
            (false, &mul, vec![0, 3, 1]),
            (true, &mul, vec![4, 0, 2]),
        ]);
    }

    if true {
        sol.set_equality(&equ);
        sol.search_all();
    } else {
        sol.set_value(&mul, &[3, 1, 1], true);
        // sol.set_value(&mul, &[0, 0, 2], true);
        sol.set_value(&mul, &[2, 2, 0], true);
        sol.set_value(&mul, &[0, 1, 3], true);
        sol.set_value(&mul, &[0, 2, 3], true);
        sol.set_value(&mul, &[0, 0, 1], false);
        sol.propagate();

        sol.evaluate_all();
        sol.print();
        sol.print_steps();
    }
}
