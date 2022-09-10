#[macro_export]
macro_rules! construct_uint {
    ($name:ident, $n_words:literal) => {
        /// Little-endian large integer type
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub struct $name(pub [u64; $n_words]);
        #[allow(unused)]
        impl $name {
            pub const ZERO: Self = $name([0; $n_words]);
            pub const MIN: Self = Self::ZERO;
            pub const MAX: Self = $name([u64::MAX; $n_words]);
            pub const BITS: u32 = $n_words * u64::BITS;
            pub const BYTES: usize = $n_words * core::mem::size_of::<u64>();
            pub const LIMBS: usize = $n_words;

            #[inline]
            pub fn from_u64(n: u64) -> Self {
                let mut ret = Self::ZERO;
                ret.0[0] = n;
                ret
            }
            #[inline]
            pub fn from_u128(n: u128) -> Self {
                let mut ret = Self::ZERO;
                ret.0[0] = n as u64;
                ret.0[1] = (n >> 64) as u64;
                ret
            }

            #[inline]
            pub fn as_u128(self) -> u128 {
                self.0[0] as u128 | ((self.0[1] as u128) << 64)
            }

            #[inline]
            pub fn as_u64(self) -> u64 {
                self.0[0] as u64
            }

            /// Return the least number of bits needed to represent the number
            #[inline]
            pub fn bits(&self) -> u32 {
                for (i, &word) in self.0.iter().enumerate().rev() {
                    if word != 0 {
                        return u64::BITS * (i as u32 + 1) - word.leading_zeros();
                    }
                }
                0
            }

            #[inline]
            pub fn overflowing_shl(self, mut s: u32) -> ($name, bool) {
                let overflows = s >= Self::BITS;
                s &= Self::BITS - 1;
                let mut ret = [0u64; $n_words];
                let left_words = (s / 64) as usize;
                let left_shifts = s % 64;

                for i in left_words..$n_words {
                    ret[i] = self.0[i - left_words] << left_shifts;
                }
                if left_shifts > 0 {
                    let left_over = 64 - left_shifts;
                    for i in left_words + 1..$n_words {
                        ret[i] |= self.0[i - 1 - left_words] >> left_over;
                    }
                }
                (Self(ret), overflows)
            }

            #[inline]
            pub fn wrapping_shl(self, s: u32) -> $name {
                self.overflowing_shl(s).0
            }

            #[inline]
            pub fn overflowing_shr(self, mut s: u32) -> ($name, bool) {
                let overflows = s >= Self::BITS;
                s &= Self::BITS - 1;
                let mut ret = [0u64; Self::LIMBS];
                let left_words = (s / 64) as usize;
                let left_shifts = s % 64;

                for i in left_words..Self::LIMBS {
                    ret[i - left_words] = self.0[i] >> left_shifts;
                }
                if left_shifts > 0 {
                    let left_over = 64 - left_shifts;
                    for i in left_words + 1..Self::LIMBS {
                        ret[i - left_words - 1] |= self.0[i] << left_over;
                    }
                }
                (Self(ret), overflows)
            }

            #[inline]
            pub fn overflowing_add(mut self, other: $name) -> ($name, bool) {
                let mut carry = false;
                let mut carry_out;
                for i in 0..Self::LIMBS {
                    (self.0[i], carry_out) = self.0[i].overflowing_add(other.0[i]);
                    self.0[i] += carry as u64; // This cannot overflow as we are adding at most 2^64 - 1 to 2^64 - 1
                    carry = carry_out;
                }
                (self, carry)
            }

            #[inline]
            pub fn overflowing_sub(mut self, other: $name) -> ($name, bool) {
                let mut carry = false;
                let mut carry_out;
                for i in 0..Self::LIMBS {
                    (self.0[i], carry_out) = self.0[i].overflowing_sub(other.0[i]);
                    self.0[i] -= carry as u64; // This cannot overflow as we are subtracting at most 2^64 - 1 from 2^64 - 1
                    carry = carry_out;
                }
                (self, carry)
            }

            /// Multiplication by u64
            #[inline]
            pub fn overflowing_mul_u64(self, other: u64) -> ($name, bool) {
                let (this, carry) = self.carrying_mul_u64(other);
                (this, carry != 0)
            }

            #[inline]
            pub fn carrying_mul_u64(mut self, other: u64) -> ($name, u64) {
                let mut carry: u128 = 0;
                for i in 0..Self::LIMBS {
                    // TODO: Use `carrying_mul` when stabilized: https://github.com/rust-lang/rust/issues/85532
                    let n = carry + (other as u128) * (self.0[i] as u128);
                    self.0[i] = n as u64;
                    carry = (n >> 64) & u64::MAX as u128;
                }
                (self, carry as u64)
            }

            #[inline]
            pub fn overflowing_mul(self, other: $name) -> ($name, bool) {
                // We should probably replace this with a Montgomery multiplication algorithm
                let mut result = $name::ZERO;
                let mut carry_out = false;
                for j in 0..Self::LIMBS {
                    let mut carry = 0;
                    let mut i = 0;
                    while i + j < Self::LIMBS {
                        let n =
                            (self.0[i] as u128) * (other.0[j] as u128) + (result.0[i + j] as u128) + (carry as u128);
                        result.0[i + j] = n as u64;
                        carry = (n >> 64) as u64;
                        i += 1;
                    }
                    carry_out |= carry != 0;
                }
                (result, carry_out)
            }
            /// Creates big integer value from a byte slice using
            /// little-endian encoding
            #[inline(always)]
            pub fn from_le_bytes(bytes: [u8; Self::BYTES]) -> $name {
                let mut out = [0u64; Self::LIMBS];
                // This should optimize to basically a transmute.
                out.iter_mut()
                    .zip(bytes.chunks_exact(8))
                    .for_each(|(word, bytes)| *word = u64::from_le_bytes(bytes.try_into().unwrap()));
                Self(out)
            }

            /// Convert's the Uint into little endian byte array
            #[inline(always)]
            pub fn to_le_bytes(self) -> [u8; Self::BYTES] {
                let mut out = [0u8; Self::BYTES];
                // This should optimize to basically a transmute.
                out.chunks_exact_mut(8)
                    .zip(self.0)
                    .for_each(|(bytes, word)| bytes.copy_from_slice(&word.to_le_bytes()));
                out
            }

            /// Convert's the Uint into big endian byte array
            #[inline(always)]
            pub fn to_be_bytes(self) -> [u8; Self::BYTES] {
                let mut out = [0u8; Self::BYTES];
                // This should optimize to basically a transmute.
                out.chunks_exact_mut(8)
                    .zip(self.0.into_iter().rev())
                    .for_each(|(bytes, word)| bytes.copy_from_slice(&word.to_be_bytes()));
                out
            }

            #[inline]
            pub fn div_rem_word(mut self, other: u64) -> (Self, u64) {
                let mut rem = 0u64;
                self.0.iter_mut().rev().for_each(|d| {
                    let n = (rem as u128) << 64 | (*d as u128);
                    *d = (n / other as u128) as u64;
                    rem = (n % other as u128) as u64;
                });
                (self, rem)
            }

            // divmod like operation, returns (quotient, remainder)
            #[inline]
            pub fn div_rem(self, other: Self) -> (Self, Self) {
                let mut sub_copy = self;
                let mut shift_copy = other;
                let mut ret = [0u64; Self::LIMBS];

                let my_bits = self.bits();
                let your_bits = other.bits();

                // Check for division by 0
                assert_ne!(your_bits, 0, "attempted to divide {:b} by zero", self);

                // Early return in case we are dividing by a larger number than us
                if my_bits < your_bits {
                    return (Self(ret), sub_copy);
                }

                // Bitwise long division
                let mut shift = my_bits - your_bits;
                shift_copy = shift_copy << shift;
                loop {
                    if sub_copy >= shift_copy {
                        let (shift_index, shift_val) = ((shift / 64) as usize, shift % 64);
                        ret[shift_index] |= 1 << shift_val;
                        sub_copy = sub_copy - shift_copy;
                    }
                    shift_copy = shift_copy >> 1;
                    if shift == 0 {
                        break;
                    }
                    shift -= 1;
                }

                (Self(ret), sub_copy)
            }

            #[inline]
            pub fn iter_be_bits(self) -> impl ExactSizeIterator<Item = bool> + core::iter::FusedIterator {
                struct BinaryIterator {
                    array: [u64; $n_words],
                    bit: usize,
                }

                impl Iterator for BinaryIterator {
                    type Item = bool;

                    #[inline]
                    fn next(&mut self) -> Option<Self::Item> {
                        if self.bit >= 64 * $n_words {
                            return None;
                        }
                        let (word, subbit) = (self.bit / 64, self.bit % 64);
                        let current_bit = self.array[$n_words - word - 1] & (1 << 64 - subbit - 1);
                        self.bit += 1;
                        Some(current_bit != 0)
                    }

                    #[inline]
                    fn nth(&mut self, n: usize) -> Option<Self::Item> {
                        match self.bit.checked_add(n) {
                            Some(bit) => {
                                self.bit = bit;
                                self.next()
                            }
                            None => {
                                self.bit = usize::MAX;
                                None
                            }
                        }
                    }
                    #[inline]
                    fn size_hint(&self) -> (usize, Option<usize>) {
                        let remaining_bits = $n_words * (u64::BITS as usize) - self.bit;
                        (remaining_bits, Some(remaining_bits))
                    }
                }
                impl ExactSizeIterator for BinaryIterator {}
                impl core::iter::FusedIterator for BinaryIterator {}

                BinaryIterator { array: self.0, bit: 0 }
            }
        }

        impl PartialEq<u64> for $name {
            #[inline]
            fn eq(&self, other: &u64) -> bool {
                let bigger = self.0[1..].iter().any(|&x| x != 0);
                !bigger && self.0[0] == *other
            }
        }
        impl PartialOrd<u64> for $name {
            #[inline]
            fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
                let bigger = self.0[1..].iter().any(|&x| x != 0);
                if bigger {
                    Some(core::cmp::Ordering::Greater)
                } else {
                    self.0[0].partial_cmp(other)
                }
            }
        }

        impl PartialEq<u128> for $name {
            #[inline]
            fn eq(&self, other: &u128) -> bool {
                let bigger = self.0[2..].iter().any(|&x| x != 0);
                !bigger && self.0[0] == (*other as u64) && self.0[1] == ((*other >> 64) as u64)
            }
        }
        impl PartialOrd<u128> for $name {
            #[inline]
            fn partial_cmp(&self, other: &u128) -> Option<core::cmp::Ordering> {
                let bigger = self.0[2..].iter().any(|&x| x != 0);
                if bigger {
                    Some(core::cmp::Ordering::Greater)
                } else {
                    self.as_u128().partial_cmp(other)
                }
            }
        }

        impl PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &$name) -> Option<core::cmp::Ordering> {
                Some(self.cmp(&other))
            }
        }

        impl Ord for $name {
            #[inline]
            fn cmp(&self, other: &$name) -> core::cmp::Ordering {
                // We need to manually implement ordering because we use little-endian
                // and the auto derive is a lexicographic ordering(i.e. memcmp)
                // which with numbers is equivalent to big-endian
                Iterator::cmp(self.0.iter().rev(), other.0.iter().rev())
            }
        }

        impl core::ops::Add<$name> for $name {
            type Output = $name;

            #[inline]
            fn add(self, other: $name) -> $name {
                let (sum, carry) = self.overflowing_add(other);
                debug_assert!(!carry, "attempt to add with overflow"); // Check in debug that it didn't overflow
                sum
            }
        }

        impl core::ops::Sub<$name> for $name {
            type Output = $name;

            #[inline]
            fn sub(self, other: $name) -> $name {
                let (sum, carry) = self.overflowing_sub(other);
                debug_assert!(!carry, "attempt to subtract with overflow"); // Check in debug that it didn't overflow
                sum
            }
        }

        impl core::ops::Mul<$name> for $name {
            type Output = $name;

            #[inline]
            fn mul(self, other: $name) -> $name {
                let (product, carry) = self.overflowing_mul(other);
                debug_assert!(!carry, "attempt to multiply with overflow"); // Check in debug that it didn't overflow
                product
            }
        }

        impl core::ops::Div<$name> for $name {
            type Output = $name;

            fn div(self, other: $name) -> $name {
                self.div_rem(other).0
            }
        }

        impl core::ops::Rem<$name> for $name {
            type Output = $name;

            fn rem(self, other: $name) -> $name {
                self.div_rem(other).1
            }
        }

        impl core::ops::BitAnd<$name> for $name {
            type Output = $name;

            #[inline]
            fn bitand(mut self, other: $name) -> $name {
                self.0
                    .iter_mut()
                    .zip(other.0.iter())
                    .for_each(|(a, b)| *a &= *b);
                self
            }
        }

        impl core::ops::BitXor<$name> for $name {
            type Output = $name;

            #[inline]
            fn bitxor(mut self, other: $name) -> $name {
                self.0
                    .iter_mut()
                    .zip(other.0.iter())
                    .for_each(|(a, b)| *a ^= *b);
                self
            }
        }

        impl core::ops::BitOr<$name> for $name {
            type Output = $name;

            #[inline]
            fn bitor(mut self, other: $name) -> $name {
                self.0
                    .iter_mut()
                    .zip(other.0.iter())
                    .for_each(|(a, b)| *a |= *b);
                self
            }
        }

        impl core::ops::Not for $name {
            type Output = $name;

            #[inline]
            fn not(mut self) -> $name {
                self.0.iter_mut().for_each(|a| *a = !*a);
                self
            }
        }

        impl core::ops::Shl<u32> for $name {
            type Output = $name;

            #[inline]
            fn shl(self, shift: u32) -> $name {
                let (res, carry) = self.overflowing_shl(shift);
                debug_assert!(!carry, "attempt to shift left with overflow"); // Check in debug that it didn't overflow
                res
            }
        }

        impl core::ops::Shr<u32> for $name {
            type Output = $name;

            #[inline]
            fn shr(self, shift: u32) -> $name {
                let (res, carry) = self.overflowing_shr(shift);
                debug_assert!(!carry, "attempt to shift left with overflow"); // Check in debug that it didn't overflow
                res
            }
        }

        impl core::fmt::LowerHex for $name {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let mut hex = [0u8; Self::BYTES * 2];
                let bytes = self.to_be_bytes();
                faster_hex::hex_encode(&bytes, &mut hex).expect("The output is exactly twice the size of the input");
                let first_non_zero = hex
                    .iter()
                    .position(|&x| x != b'0')
                    .unwrap_or(hex.len() - 1);
                // The string is hex encoded so must be valid UTF8.
                let str = unsafe { core::str::from_utf8_unchecked(&hex[first_non_zero..]) };
                f.pad_integral(true, "0x", str)
            }
        }

        // Based on https://github.com/rust-lang/rust/blob/2e44c17c12cec45b6a682b1e53a04ac5b5fcc9d2/library/core/src/fmt/num.rs#L209
        impl core::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                // 2 digit decimal look up table
                static DEC_DIGITS_LUT: &[u8; 200] = b"0001020304050607080910111213141516171819\
            2021222324252627282930313233343536373839\
            4041424344454647484950515253545556575859\
            6061626364656667686970717273747576777879\
            8081828384858687888990919293949596979899";

                let mut buf = [0u8; $name::LIMBS * 20]; // 2**64-1 takes 20 digits to represent.
                let mut n = *self;
                let mut curr = buf.len();

                // eagerly decode 4 characters at a time
                const STEP: u64 = 10_000;
                while n >= STEP {
                    let rem: u64;
                    (n, rem) = n.div_rem_word(STEP);
                    let rem = rem as usize;
                    let d1 = (rem / 100) << 1;
                    let d2 = (rem % 100) << 1;
                    curr -= 4;

                    buf[curr] = DEC_DIGITS_LUT[d1];
                    buf[curr + 1] = DEC_DIGITS_LUT[d1 + 1];
                    buf[curr + 2] = DEC_DIGITS_LUT[d2];
                    buf[curr + 3] = DEC_DIGITS_LUT[d2 + 1];
                }
                // if we reach here numbers are <= 9999, so at most 4 chars long
                let mut n = n.as_u64() as usize; // possibly reduce 64bit math

                // decode 2 more chars, if > 2 chars
                if n >= 100 {
                    let d1 = (n % 100) << 1;
                    n /= 100;
                    curr -= 2;
                    buf[curr] = DEC_DIGITS_LUT[d1 as usize];
                    buf[curr + 1] = DEC_DIGITS_LUT[d1 + 1 as usize];
                }

                // decode last 1 or 2 chars
                if n < 10 {
                    curr -= 1;
                    buf[curr] = (n as u8) + b'0'
                } else {
                    let d1 = n << 1;
                    curr -= 2;
                    buf[curr] = DEC_DIGITS_LUT[d1];
                    buf[curr + 1] = DEC_DIGITS_LUT[d1 + 1];
                }

                // SAFETY: everything up to `curr` is valid UTF8 because `DEC_DIGITS_LUT` is.
                let buf_str = unsafe { std::str::from_utf8_unchecked(&buf[curr..]) };
                f.pad_integral(true, "", buf_str)
            }
        }

        impl core::fmt::Binary for $name {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                const BIN_LEN: usize = $name::BITS as usize;
                let mut buf = [0u8; BIN_LEN];
                let mut first_one = BIN_LEN - 1;
                for (index, (bit, char)) in self
                    .iter_be_bits()
                    .zip(buf.iter_mut())
                    .enumerate()
                {
                    *char = bit as u8 + b'0';
                    if first_one == BIN_LEN - 1 && bit {
                        first_one = index;
                    }
                }
                // We only wrote '0' and '1' so this is always valid UTF-8
                let buf_str = unsafe { std::str::from_utf8_unchecked(&buf[first_one..]) };
                f.pad_integral(true, "0b", buf_str)
            }
        }

        // We can't derive because the array might be bigger than 32,
        // so we just implement it the same as arrays.
        impl serde::Serialize for $name {
            #[inline]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeTuple;
                let mut seq = serializer.serialize_tuple(Self::LIMBS)?;
                for limb in &self.0 {
                    seq.serialize_element(limb)?;
                }
                seq.end()
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            #[inline]
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                use core::{fmt, marker::PhantomData};
                use serde::de::{Error, SeqAccess, Visitor};
                struct EmptyVisitor(PhantomData<$name>);
                impl<'de> Visitor<'de> for EmptyVisitor {
                    type Value = $name;
                    #[inline]

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(concat!("an integer with ", $n_words, " limbs"))
                    }

                    #[inline]
                    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                        let mut ret = $name::ZERO;
                        for (i, limb) in ret.0.iter_mut().enumerate() {
                            *limb = seq
                                .next_element()?
                                .ok_or_else(|| Error::invalid_length(i, &self))?;
                        }
                        Ok(ret)
                    }
                }
                deserializer.deserialize_tuple(Self::LIMBS, EmptyVisitor(PhantomData))
            }

            #[inline]
            fn deserialize_in_place<D: serde::Deserializer<'de>>(
                deserializer: D, place: &mut Self,
            ) -> Result<(), D::Error> {
                use core::fmt;
                use serde::de::{Error, SeqAccess, Visitor};
                struct InPlaceVisitor<'a>(&'a mut $name);

                impl<'de, 'a> Visitor<'de> for InPlaceVisitor<'a> {
                    type Value = ();
                    #[inline]
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(concat!("an integer with ", $n_words, " limbs"))
                    }
                    #[inline]
                    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                        for (idx, dest) in self.0 .0[..].iter_mut().enumerate() {
                            match seq.next_element()? {
                                Some(elem) => *dest = elem,
                                None => {
                                    return Err(Error::invalid_length(idx, &self));
                                }
                            }
                        }
                        Ok(())
                    }
                }
                deserializer.deserialize_tuple(Self::LIMBS, InPlaceVisitor(place))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use rand_chacha::{
        rand_core::{RngCore, SeedableRng},
        ChaCha8Rng,
    };
    use std::fmt::Write;
    construct_uint!(Uint128, 2);

    #[test]
    fn test_u128() {
        use core::fmt::Arguments;
        let mut fmt_buf = String::with_capacity(256);
        let mut fmt_buf2 = String::with_capacity(256);
        let mut assert_equal_args = |arg1: Arguments, arg2: Arguments| {
            fmt_buf.clear();
            fmt_buf2.clear();
            fmt_buf.write_fmt(arg1).unwrap();
            fmt_buf2.write_fmt(arg2).unwrap();
            assert_eq!(fmt_buf, fmt_buf2);
        };
        let mut assert_equal = |a: Uint128, b: u128, check_fmt: bool| {
            assert_eq!(a, b);
            assert_eq!(a.to_le_bytes(), b.to_le_bytes());
            if !check_fmt {
                return;
            }

            assert_equal_args(format_args!("{a:}"), format_args!("{b:}"));
            assert_equal_args(format_args!("{a:b}"), format_args!("{b:b}")); // Test Binary
            assert_equal_args(format_args!("{a:#b}"), format_args!("{b:#b}")); // Test Binary with prefix
            assert_equal_args(format_args!("{a:0128b}"), format_args!("{b:0128b}")); // Test binary with length
            assert_equal_args(format_args!("{a:x}"), format_args!("{b:x}")); // Test LowerHex
            assert_equal_args(format_args!("{a:#x}"), format_args!("{b:#x}")); // Test LowerHex with prefix
            // Test LowerHex with padding
            assert_equal_args(format_args!("{a:0256x}"), format_args!("{b:0256x}"));
        };
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let mut buf = [0u8; 16];
        for i in 0..64_000 {
            // Checking all the fmt's is quite expensive.
            let check_fmt = i % 8 == 1;
            rng.fill_bytes(&mut buf);
            let mine = Uint128::from_le_bytes(buf);
            let default = u128::from_le_bytes(buf);
            rng.fill_bytes(&mut buf);
            let mine2 = Uint128::from_le_bytes(buf);
            let default2 = u128::from_le_bytes(buf);
            assert_equal(mine, default, check_fmt);
            assert_equal(mine2, default2, check_fmt);

            let mine = mine
                .overflowing_add(mine2)
                .0
                .overflowing_mul(mine2)
                .0;
            let default = default
                .overflowing_add(default2)
                .0
                .overflowing_mul(default2)
                .0;
            assert_equal(mine, default, check_fmt);
            let shift = rng.next_u32() % 4096;
            {
                let mine_overflow_shl = mine.overflowing_shl(shift);
                let default_overflow_shl = default.overflowing_shl(shift);
                assert_equal(mine_overflow_shl.0, default_overflow_shl.0, check_fmt);
                assert_eq!(mine_overflow_shl.1, default_overflow_shl.1);
            }
            {
                let mine_overflow_shr = mine.overflowing_shl(shift);
                let default_overflow_shr = default.overflowing_shl(shift);
                assert_equal(mine_overflow_shr.0, default_overflow_shr.0, check_fmt);
                assert_eq!(mine_overflow_shr.1, default_overflow_shr.1);
            }
            {
                let mine_divrem = mine.div_rem(mine2);
                let default_divrem = (default / default2, default % default2);
                assert_equal(mine_divrem.0, default_divrem.0, check_fmt);
                assert_equal(mine_divrem.1, default_divrem.1, check_fmt);
            }
            // Test fast u64 division.
            {
                let rand_u64 = rng.next_u64();
                let mine_divrem = mine.div_rem_word(rand_u64);
                let default_divrem = (default / u128::from(rand_u64), default % u128::from(rand_u64));
                assert_equal(mine_divrem.0, default_divrem.0, check_fmt);
                assert_eq!(mine_divrem.1, u64::try_from(default_divrem.1).unwrap());
            }
        }
    }
}