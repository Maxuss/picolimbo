use std::fmt::Display;

macro_rules! build_protocol {
    ($(
        $variant:ident = $idx:literal
    ),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, serde::Deserialize)]
        #[repr(i16)]
        #[serde(rename_all = "snake_case")]
        pub enum Protocol {
            $(
                $variant = $idx
            ),*
        }

        impl Protocol {
            pub const fn latest() -> Self {
                Self::V1_19_4
            }

            pub const fn from_idx(idx: i32) -> Self {
                match idx {
                    $(
                    $idx => Self::$variant,
                    )*
                    _ => Self::Legacy
                }
            }
        }

        impl Display for Protocol {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unreachable_patterns)]
                let raw_st = match self {
                    Self::Legacy => "unsupported",
                    $(
                        Self::$variant => stringify!($variant),
                    )*
                };
                f.write_str(&raw_st.replace("_", ".").replace("V", ""))
            }
        }
    };
}

build_protocol! {
    Legacy = -1,
    V1_7_2 = 4,
    V1_7_6 = 5,
    V1_8 = 47,
    V1_9 = 107,
    V1_9_1 = 108,
    V1_9_2 = 109,
    V1_9_4 = 110,
    V1_10 = 210,
    // 1.10-1.10.2 has same protocol numbers
    V1_11 = 315,
    V1_11_1 = 316,
    // 1.11.2 has same protocol number
    V1_12 = 335,
    V1_12_1 = 338,
    V1_12_2 = 340,
    V1_13 = 393,
    V1_13_1 = 401,
    V1_13_2 = 404,
    V1_14 = 477,
    V1_14_1 = 480,
    V1_14_2 = 485,
    V1_14_3 = 490,
    V1_14_4 = 498,
    V1_15 = 573,
    V1_15_1 = 575,
    V1_15_2 = 578,
    V1_16 = 735,
    V1_16_1 = 736,
    V1_16_2 = 751,
    V1_16_3 = 753,
    V1_16_4 = 754,
    // 1.16.5 has same protocol number
    V1_17 = 755,
    V1_17_1 = 756,
    V1_18 = 757,
    // 1.18.1 has same protocol number
    V1_18_2 = 758,
    V1_19 = 759,
    V1_19_1 = 760,
    // 1.19.2 has same protocol number
    V1_19_3 = 761,
    V1_19_4 = 762,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::latest()
    }
}
