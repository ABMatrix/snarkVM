// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    function::{parsers::*, Instruction, Opcode, Operation, Register, Registers},
    Program,
    Value,
};
use snarkvm_circuits::{
    count,
    Count,
    Field,
    Literal,
    LiteralType,
    Metrics,
    MulChecked,
    Parser,
    ParserResult,
    I128,
    I16,
    I32,
    I64,
    I8,
    U128,
    U16,
    U32,
    U64,
    U8,
};
use snarkvm_utilities::{FromBytes, ToBytes};

use core::{fmt, ops::Mul as MulCircuit};
use nom::combinator::map;
use std::io::{Read, Result as IoResult, Write};

/// Multiplies `first` and `second`, storing the outcome in `destination`.
pub struct Mul<P: Program> {
    operation: BinaryOperation<P>,
}

impl<P: Program> Mul<P> {
    /// Returns the operands of the instruction.
    pub fn operands(&self) -> Vec<Operand<P>> {
        self.operation.operands()
    }

    /// Returns the destination register of the instruction.
    pub fn destination(&self) -> &Register<P> {
        self.operation.destination()
    }
}

impl<P: Program> Opcode for Mul<P> {
    /// Returns the opcode as a string.
    #[inline]
    fn opcode() -> &'static str {
        "mul"
    }
}

impl<P: Program> Operation<P> for Mul<P> {
    /// Evaluates the operation.
    #[inline]
    fn evaluate(&self, registers: &Registers<P>) {
        // Load the values for the first and second operands.
        let first = match registers.load(self.operation.first()) {
            Value::Literal(literal) => literal,
            Value::Definition(name, ..) => P::halt(format!("{name} is not a literal")),
        };
        let second = match registers.load(self.operation.second()) {
            Value::Literal(literal) => literal,
            Value::Definition(name, ..) => P::halt(format!("{name} is not a literal")),
        };

        // Perform the operation.
        let result = match (first, second) {
            (Literal::Field(a), Literal::Field(b)) => Literal::Field(a * b),
            (Literal::Group(a), Literal::Scalar(b)) => Literal::Group(a * b),
            (Literal::Scalar(a), Literal::Group(b)) => Literal::Group(a * b),
            (Literal::I8(a), Literal::I8(b)) => Literal::I8(a.mul_checked(&b)),
            (Literal::I16(a), Literal::I16(b)) => Literal::I16(a.mul_checked(&b)),
            (Literal::I32(a), Literal::I32(b)) => Literal::I32(a.mul_checked(&b)),
            (Literal::I64(a), Literal::I64(b)) => Literal::I64(a.mul_checked(&b)),
            (Literal::I128(a), Literal::I128(b)) => Literal::I128(a.mul_checked(&b)),
            (Literal::U8(a), Literal::U8(b)) => Literal::U8(a.mul_checked(&b)),
            (Literal::U16(a), Literal::U16(b)) => Literal::U16(a.mul_checked(&b)),
            (Literal::U32(a), Literal::U32(b)) => Literal::U32(a.mul_checked(&b)),
            (Literal::U64(a), Literal::U64(b)) => Literal::U64(a.mul_checked(&b)),
            (Literal::U128(a), Literal::U128(b)) => Literal::U128(a.mul_checked(&b)),
            _ => P::halt(format!("Invalid '{}' instruction", Self::opcode())),
        };

        registers.assign(self.operation.destination(), result);
    }
}

impl<P: Program> Metrics<Self> for Mul<P> {
    type Case = (LiteralType<P::Environment>, LiteralType<P::Environment>);

    fn count(case: &Self::Case) -> Count {
        crate::match_count!(match MulCircuit::count(case) {
            (Field, Field) => Field,
            // (Group, Scalar) => Group,
            // (Scalar, Group) => Group,
            (I8, I8) => I8,
            (I16, I16) => I16,
            (I32, I32) => I32,
            (I64, I64) => I64,
            (I128, I128) => I128,
            (U8, U8) => U8,
            (U16, U16) => U16,
            (U32, U32) => U32,
            (U64, U64) => U64,
            (U128, U128) => U128,
        })
    }
}

impl<P: Program> Parser for Mul<P> {
    type Environment = P::Environment;

    /// Parses a string into a 'mul' operation.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Parse the operation from the string.
        let (string, operation) = map(BinaryOperation::parse, |operation| Self { operation })(string)?;
        // Return the operation.
        Ok((string, operation))
    }
}

impl<P: Program> fmt::Display for Mul<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operation)
    }
}

impl<P: Program> FromBytes for Mul<P> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Ok(Self { operation: BinaryOperation::read_le(&mut reader)? })
    }
}

impl<P: Program> ToBytes for Mul<P> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.operation.write_le(&mut writer)
    }
}

#[allow(clippy::from_over_into)]
impl<P: Program> Into<Instruction<P>> for Mul<P> {
    /// Converts the operation into an instruction.
    fn into(self) -> Instruction<P> {
        Instruction::Mul(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{function::Register, test_instruction_halts, test_modes, Identifier, Process};

    type P = Process;

    #[test]
    fn test_parse() {
        let (_, instruction) = Instruction::<Process>::parse("mul r0 r1 into r2;").unwrap();
        assert!(matches!(instruction, Instruction::Mul(_)));
    }

    // Testing this manually since the constant x public mode yields a public,
    // but the test_modes! macro expects a private.
    // test_modes!(field, Mul, "1field", "2field", "2field");
    mod field {
        use super::Mul;
        use crate::binary_instruction_test;
        binary_instruction_test!(
            test_public_and_public_yields_private,
            Mul,
            "1field.public",
            "2field.public",
            "2field.private"
        );

        binary_instruction_test!(
            test_public_and_constant_yields_private,
            Mul,
            "1field.public",
            "2field.constant",
            "2field.private"
        );

        binary_instruction_test!(
            test_public_and_private_yields_private,
            Mul,
            "1field.public",
            "2field.private",
            "2field.private"
        );

        binary_instruction_test!(
            test_private_and_constant_yields_private,
            Mul,
            "1field.private",
            "2field.constant",
            "2field.private"
        );

        binary_instruction_test!(
            test_private_and_public_yields_private,
            Mul,
            "1field.private",
            "2field.public",
            "2field.private"
        );

        binary_instruction_test!(
            test_private_and_private_yields_private,
            Mul,
            "1field.private",
            "2field.private",
            "2field.private"
        );

        binary_instruction_test!(
            test_constant_and_private_yields_private,
            Mul,
            "1field.constant",
            "2field.private",
            "2field.private"
        );

        binary_instruction_test!(
            test_constant_and_public_yields_public,
            Mul,
            "1field.constant",
            "2field.public",
            "2field.public"
        );

        binary_instruction_test!(
            test_constant_and_constant_yields_constant,
            Mul,
            "1field.constant",
            "2field.constant",
            "2field.constant"
        );
    }

    mod group {
        use super::*;
        use crate::binary_instruction_test;

        // 2group * 1scalar has special output mode behavior.
        // Normally, a public variable times a constant would yield a private variable. However, since
        // the constant is one, we return the original public variable.
        binary_instruction_test!(
            constant_and_constant_yields_constant,
            Mul,
            "2group.constant",
            "1scalar.constant",
            "2group.constant"
        );
        binary_instruction_test!(
            constant_and_public_yields_private,
            Mul,
            "2group.constant",
            "1scalar.public",
            "2group.private"
        );
        binary_instruction_test!(
            constant_and_private_yields_private,
            Mul,
            "2group.constant",
            "1scalar.private",
            "2group.private"
        );
        binary_instruction_test!(
            public_and_constant_yields_public,
            Mul,
            "2group.public",
            "1scalar.constant",
            "2group.public"
        );
        binary_instruction_test!(
            private_and_constant_yields_private,
            Mul,
            "2group.private",
            "1scalar.constant",
            "2group.private"
        );
        binary_instruction_test!(
            public_and_public_yields_private,
            Mul,
            "2group.public",
            "1scalar.public",
            "2group.private"
        );
        binary_instruction_test!(
            public_and_private_yields_private,
            Mul,
            "2group.public",
            "1scalar.private",
            "2group.private"
        );
        binary_instruction_test!(
            private_and_public_yields_private,
            Mul,
            "2group.private",
            "1scalar.public",
            "2group.private"
        );
        binary_instruction_test!(
            private_and_private_yields_private,
            Mul,
            "2group.private",
            "1scalar.private",
            "2group.private"
        );
    }

    mod scalar {
        use super::*;
        use crate::binary_instruction_test;

        // 1scalar * 2group has special output mode behavior.
        // Normally, a constant times a public variable would yield a private variable. However, since
        // the constant is one, we return the original public variable.
        binary_instruction_test!(
            constant_and_constant_yields_constant,
            Mul,
            "1scalar.constant",
            "2group.constant",
            "2group.constant"
        );
        binary_instruction_test!(
            constant_and_public_yields_private,
            Mul,
            "1scalar.constant",
            "2group.public",
            "2group.public"
        );
        binary_instruction_test!(
            constant_and_private_yields_private,
            Mul,
            "1scalar.constant",
            "2group.private",
            "2group.private"
        );
        binary_instruction_test!(
            public_and_constant_yields_public,
            Mul,
            "1scalar.public",
            "2group.constant",
            "2group.private"
        );
        binary_instruction_test!(
            private_and_constant_yields_private,
            Mul,
            "1scalar.private",
            "2group.constant",
            "2group.private"
        );
        binary_instruction_test!(
            public_and_public_yields_private,
            Mul,
            "1scalar.public",
            "2group.public",
            "2group.private"
        );
        binary_instruction_test!(
            public_and_private_yields_private,
            Mul,
            "1scalar.public",
            "2group.private",
            "2group.private"
        );
        binary_instruction_test!(
            private_and_public_yields_private,
            Mul,
            "1scalar.private",
            "2group.public",
            "2group.private"
        );
        binary_instruction_test!(
            private_and_private_yields_private,
            Mul,
            "1scalar.private",
            "2group.private",
            "2group.private"
        );
    }

    test_modes!(i8, Mul, "1i8", "2i8", "2i8");
    test_modes!(i16, Mul, "1i16", "2i16", "2i16");
    test_modes!(i32, Mul, "1i32", "2i32", "2i32");
    test_modes!(i64, Mul, "1i64", "2i64", "2i64");
    test_modes!(i128, Mul, "1i128", "2i128", "2i128");
    test_modes!(u8, Mul, "1u8", "2u8", "2u8");
    test_modes!(u16, Mul, "1u16", "2u16", "2u16");
    test_modes!(u32, Mul, "1u32", "2u32", "2u32");
    test_modes!(u64, Mul, "1u64", "2u64", "2u64");
    test_modes!(u128, Mul, "1u128", "2u128", "2u128");

    test_instruction_halts!(
        i8_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}i8.constant", i8::MAX),
        "2i8.constant"
    );
    test_instruction_halts!(
        i16_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}i16.constant", i16::MAX),
        "2i16.constant"
    );
    test_instruction_halts!(
        i32_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}i32.constant", i32::MAX),
        "2i32.constant"
    );
    test_instruction_halts!(
        i64_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}i64.constant", i64::MAX),
        "2i64.constant"
    );
    test_instruction_halts!(
        i128_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}i128.constant", i128::MAX),
        "2i128.constant"
    );
    test_instruction_halts!(
        u8_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}u8.constant", u8::MAX),
        "2u8.constant"
    );
    test_instruction_halts!(
        u16_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}u16.constant", u16::MAX),
        "2u16.constant"
    );
    test_instruction_halts!(
        u32_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}u32.constant", u32::MAX),
        "2u32.constant"
    );
    test_instruction_halts!(
        u64_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}u64.constant", u64::MAX),
        "2u64.constant"
    );
    test_instruction_halts!(
        u128_overflow_halts,
        Mul,
        "Integer overflow on multiplication of two constants",
        &format!("{}u128.constant", u128::MAX),
        "2u128.constant"
    );

    test_instruction_halts!(
        address_halts,
        Mul,
        "Invalid 'mul' instruction",
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah.constant",
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah.constant"
    );
    test_instruction_halts!(boolean_halts, Mul, "Invalid 'mul' instruction", "true.constant", "true.constant");
    test_instruction_halts!(string_halts, Mul, "Invalid 'mul' instruction", "\"hello\".constant", "\"world\".constant");

    #[test]
    #[should_panic(expected = "message is not a literal")]
    fn test_definition_halts() {
        let first = Value::<P>::Definition(Identifier::from_str("message"), vec![
            Value::from_str("2group.public"),
            Value::from_str("10field.private"),
        ]);
        let second = first.clone();

        let registers = Registers::<P>::default();
        registers.define(&Register::from_str("r0"));
        registers.define(&Register::from_str("r1"));
        registers.define(&Register::from_str("r2"));
        registers.assign(&Register::from_str("r0"), first);
        registers.assign(&Register::from_str("r1"), second);

        Mul::from_str("r0 r1 into r2").evaluate(&registers);
    }
}
