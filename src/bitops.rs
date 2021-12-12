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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bit1(u32);

impl Bit1 {
    #[inline(always)]
    pub fn new(val: u32) -> Bit1 {
        debug_assert!(val <= 1);
        Bit1(val)
    }

    #[inline(always)]
    pub fn idx(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bit2(u32);

impl Bit2 {
    #[inline(always)]
    pub const fn new(val: u32) -> Bit2 {
        debug_assert!(val <= 3);
        Bit2(val)
    }

    #[inline(always)]
    pub const fn idx(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Op22(u32);

impl Op22 {
    pub const fn new(cases: &[(Bit2, Bit2)]) -> Self {
        assert!(cases.len() == 4);
        let mut set: u32 = 0;
        let mut val: u32 = 0;
        let mut idx = 0;
        while idx < cases.len() {
            let (a, b) = cases[idx];
            assert!(a.0 <= 3 && b.0 <= 3);
            let pos = a.0 << 1;
            val |= b.0 << pos;
            set |= 3 << pos;
            idx += 1;
        }
        assert!(set == 0xff);
        Op22(val)
    }

    #[inline(always)]
    pub const fn of(self, a: Bit2) -> Bit2 {
        Bit2((self.0 >> (a.0 << 1)) & 3)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Op222(u32);

impl Op222 {
    pub const fn new(cases: &[(Bit2, Bit2, Bit2)]) -> Self {
        assert!(cases.len() == 16);
        let mut set: u32 = 0;
        let mut val: u32 = 0;
        let mut idx = 0;
        while idx < cases.len() {
            let (a, b, c) = cases[idx];
            assert!(a.0 <= 3 && b.0 <= 3 && c.0 <= 3);
            let pos = (a.0 << 3) | (b.0 << 1);
            val |= c.0 << pos;
            set |= 3 << pos;
            idx += 1;
        }
        assert!(set == 0xffffffff);
        Op222(val)
    }

    #[inline(always)]
    pub const fn of(self, a: Bit2, b: Bit2) -> Bit2 {
        Bit2((self.0 >> ((a.0 << 3) | (b.0 << 1))) & 3)
    }

    #[cfg(test)]
    fn idempotent(self) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            if self.of(a, a) != a {
                return false;
            }
        }
        true
    }

    #[cfg(test)]
    fn commutative(self) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                if self.of(a, b) != self.of(b, a) {
                    return false;
                }
            }
        }
        true
    }

    #[cfg(test)]
    fn associative(self) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                for c in 0..3 {
                    let c = Bit2(c);
                    if self.of(self.of(a, b), c) != self.of(a, self.of(b, c)) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

pub const BOOL_FALSE: Bit2 = Bit2(0);
pub const BOOL_UNDEF: Bit2 = Bit2(1);
pub const BOOL_TRUE: Bit2 = Bit2(2);
pub const BOOL_MISSING: Bit2 = Bit2(3);

pub const BOOL_FORMAT: [char; 4] = ['0', '?', '1', 'x'];

pub const BOOL_NOT: Op22 = Op22::new(&[
    (BOOL_FALSE, BOOL_TRUE),
    (BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_FALSE),
    (BOOL_MISSING, BOOL_MISSING),
]);

pub const BOOL_OR: Op222 = Op222::new(&[
    (BOOL_FALSE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_FALSE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_FALSE, BOOL_MISSING, BOOL_FALSE),
    (BOOL_UNDEF, BOOL_FALSE, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_TRUE, BOOL_TRUE),
    (BOOL_UNDEF, BOOL_MISSING, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_FALSE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_UNDEF, BOOL_TRUE),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_MISSING, BOOL_TRUE),
    (BOOL_MISSING, BOOL_FALSE, BOOL_FALSE),
    (BOOL_MISSING, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_MISSING, BOOL_TRUE, BOOL_TRUE),
    (BOOL_MISSING, BOOL_MISSING, BOOL_MISSING),
]);

pub const BOOL_AND: Op222 = Op222::new(&[
    (BOOL_FALSE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF, BOOL_FALSE),
    (BOOL_FALSE, BOOL_TRUE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_MISSING, BOOL_FALSE),
    (BOOL_UNDEF, BOOL_FALSE, BOOL_FALSE),
    (BOOL_UNDEF, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_TRUE, BOOL_UNDEF),
    (BOOL_UNDEF, BOOL_MISSING, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_TRUE, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_MISSING, BOOL_TRUE),
    (BOOL_MISSING, BOOL_FALSE, BOOL_FALSE),
    (BOOL_MISSING, BOOL_UNDEF, BOOL_UNDEF),
    (BOOL_MISSING, BOOL_TRUE, BOOL_TRUE),
    (BOOL_MISSING, BOOL_MISSING, BOOL_MISSING),
]);

pub const EVAL_FALSE: Bit2 = Bit2(0);
pub const EVAL_UNIT: Bit2 = Bit2(1);
pub const EVAL_UNDEF: Bit2 = Bit2(2);
pub const EVAL_TRUE: Bit2 = Bit2(3);

pub const EVAL_FORMAT1: [char; 4] = ['0', '!', '?', '1'];
pub const EVAL_FORMAT2: [&str; 4] = ["false", "unit", "undef", "true"];

pub const FOLD_POS: Op222 = Op222::new(&[
    (EVAL_FALSE, BOOL_FALSE, EVAL_FALSE),
    (EVAL_FALSE, BOOL_UNDEF, EVAL_UNIT),
    (EVAL_FALSE, BOOL_TRUE, EVAL_TRUE),
    (EVAL_FALSE, BOOL_MISSING, EVAL_FALSE),
    (EVAL_UNIT, BOOL_FALSE, EVAL_UNIT),
    (EVAL_UNIT, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNIT, BOOL_TRUE, EVAL_TRUE),
    (EVAL_UNIT, BOOL_MISSING, EVAL_UNIT),
    (EVAL_UNDEF, BOOL_FALSE, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_TRUE, EVAL_TRUE),
    (EVAL_UNDEF, BOOL_MISSING, EVAL_UNDEF),
    (EVAL_TRUE, BOOL_FALSE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_UNDEF, EVAL_TRUE),
    (EVAL_TRUE, BOOL_TRUE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_MISSING, EVAL_TRUE),
]);

pub const FOLD_NEG: Op222 = Op222::new(&[
    (EVAL_FALSE, BOOL_FALSE, EVAL_TRUE),
    (EVAL_FALSE, BOOL_UNDEF, EVAL_UNIT),
    (EVAL_FALSE, BOOL_TRUE, EVAL_FALSE),
    (EVAL_FALSE, BOOL_MISSING, EVAL_FALSE),
    (EVAL_UNIT, BOOL_FALSE, EVAL_TRUE),
    (EVAL_UNIT, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNIT, BOOL_TRUE, EVAL_UNIT),
    (EVAL_UNIT, BOOL_MISSING, EVAL_UNIT),
    (EVAL_UNDEF, BOOL_FALSE, EVAL_TRUE),
    (EVAL_UNDEF, BOOL_UNDEF, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_TRUE, EVAL_UNDEF),
    (EVAL_UNDEF, BOOL_MISSING, EVAL_UNDEF),
    (EVAL_TRUE, BOOL_FALSE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_UNDEF, EVAL_TRUE),
    (EVAL_TRUE, BOOL_TRUE, EVAL_TRUE),
    (EVAL_TRUE, BOOL_MISSING, EVAL_TRUE),
]);

pub const EVAL_AND: Op222 = Op222::new(&[
    (EVAL_FALSE, EVAL_FALSE, EVAL_FALSE),
    (EVAL_FALSE, EVAL_UNIT, EVAL_FALSE),
    (EVAL_FALSE, EVAL_UNDEF, EVAL_FALSE),
    (EVAL_FALSE, EVAL_TRUE, EVAL_FALSE),
    (EVAL_UNIT, EVAL_FALSE, EVAL_FALSE),
    (EVAL_UNIT, EVAL_UNIT, EVAL_UNIT),
    (EVAL_UNIT, EVAL_UNDEF, EVAL_UNIT),
    (EVAL_UNIT, EVAL_TRUE, EVAL_UNIT),
    (EVAL_UNDEF, EVAL_FALSE, EVAL_FALSE),
    (EVAL_UNDEF, EVAL_UNIT, EVAL_UNIT),
    (EVAL_UNDEF, EVAL_UNDEF, EVAL_UNDEF),
    (EVAL_UNDEF, EVAL_TRUE, EVAL_UNDEF),
    (EVAL_TRUE, EVAL_FALSE, EVAL_FALSE),
    (EVAL_TRUE, EVAL_UNIT, EVAL_UNIT),
    (EVAL_TRUE, EVAL_UNDEF, EVAL_UNDEF),
    (EVAL_TRUE, EVAL_TRUE, EVAL_TRUE),
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laws() {
        assert!(BOOL_AND.idempotent());
        assert!(BOOL_AND.commutative());
        assert!(BOOL_AND.associative());

        assert!(BOOL_OR.idempotent());
        assert!(BOOL_OR.commutative());
        assert!(BOOL_OR.associative());

        assert!(EVAL_AND.idempotent());
        assert!(EVAL_AND.commutative());
        assert!(EVAL_AND.associative());

        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                assert_eq!(FOLD_POS.of(a, b), FOLD_NEG.of(a, BOOL_NOT.of(b)));
            }
        }

        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                for c in 0..3 {
                    let c = Bit2(c);
                    assert_eq!(
                        FOLD_POS.of(FOLD_POS.of(a, b), c),
                        FOLD_POS.of(FOLD_POS.of(a, c), b),
                    );
                }
            }
        }
    }
}
