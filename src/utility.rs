pub mod internal {
    macro_rules! bitset_bits_impl {
        ($vis:tt, $typename:tt, $type:tt, $bit:literal : $name_lower:ident => $name_upper:ident; $($rest:tt)*) => {
            $vis const $name_upper: $type = 1 << $bit;

            paste::paste! {
                $vis fn [<get_ $name_lower>](&self) -> bool {
                    (self.0 & (1 << $bit)) != 0
                }

                $vis fn [<with_ $name_lower>](&self, value: bool) -> Self {
                    if value {
                        Self { 0: self.0 | Self::$name_upper } // set
                    } else {
                        Self { 0: self.0 & !Self::$name_upper } // clear
                    }
                }
            }

            $crate::utility::internal::bitset_bits_impl!($vis, $typename, $type, $($rest)*);
        };
        ($($rest:tt)*) => {};
    }
    pub(crate) use bitset_bits_impl;
}

/// Bitset provides a convinient way to define bit flags represented
/// by an unsigned numeric type.
macro_rules! bitset {
    ($(#[$attribute:meta])* $vis:vis struct $name:ident($type:ty); $($rest:tt)*) => {
        $(#[$attribute])*
        $vis struct $name($type);

        #[allow(dead_code)]
        impl $name {
            $vis fn new() -> Self {
                Self { 0: 0 }
            }

            $vis fn from(state: $type) -> Self {
                Self { 0: state }
            }

            //

            $vis fn get_raw(&self) -> $type {
                self.0
            }

            $vis fn set_raw(&mut self, raw: $type) {
                self.0 = raw;
            }

            $vis fn get(&self, mask: $type) -> $type {
                self.0 & mask
            }

            $vis fn set(&mut self, mask: $type) {
                self.0 |= mask;
            }

            $vis fn replace(&mut self, other: Self) {
                self.0 = other.0
            }

            $crate::utility::internal::bitset_bits_impl!($vis, $name, $type, $($rest)*);
        }
    };
    ($($rest:tt)*) => {
        compile_error!("invalid bitset specification");
    };
}
pub(crate) use bitset;

macro_rules! combine_literals {
    ($sep:literal | $lit1:literal $lit2:literal $($rest:tt)*) => {
        $crate::utility::combine_literals!(join: $sep | $lit1 $lit2 $($rest)*)
    };

    (join: $sep:literal | $lit1:literal $lit2:literal $($rest:tt)*) => {
        concat!($lit1, $sep, $lit2, $crate::utility::combine_literals!(join: $sep | $($rest)*))
    };
    (join: $sep:literal | $lit:literal $($rest:tt)*) => {
        concat!($sep, $lit)
    };
    (join: $sep:literal |) => { "" };

    (prefix: $prefix:literal | $lit:literal $($rest:tt)*) => {
        concat!($prefix, $lit, $crate::utility::combine_literals!(prefix: $prefix | $($rest)*))
    };
    (prefix: $prefix:literal |) => { "" };

    (suffix: $suffix:literal | $lit:literal $($rest:tt)*) => {
        concat!($lit, $suffix, $crate::utility::combine_literals!(suffix: $suffix | $($rest)*))
    };
    (suffix: $suffix:literal |) => { "" };
}
pub(crate) use combine_literals;

/// Multiline enables you to cleanly define multi-line strings.
macro_rules! multiline {
    ($($lines:literal)*) => {
        $crate::utility::combine_literals!(join: "\n" | $($lines)*)
    }
}
pub(crate) use multiline;

macro_rules! overline {
    ($($chars:literal)*) => {
        $crate::utility::combine_literals!(suffix: '\u{0305}' | $($chars)*)
    };
}
pub(crate) use overline;
