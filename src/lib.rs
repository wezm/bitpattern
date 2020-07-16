use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;

/// bitwise pattern matching and extracting.
///
/// # Example
///
///```rust
///use bitmatch::bitmatch;
///
///let x = 0xacu8; // 10101100
///
///// '0' means the bit must be 0.
///// '1' means the bit must be 1.
///// ' ' can be uses as separator.
///assert_eq!(bitmatch!("1010 1100", x), Some(()));
///assert_eq!(bitmatch!("1010 0100", x), None);
///
///// '_' means the bit can be 0 or 1.
///assert_eq!(bitmatch!("1_10 1_00", x), Some(()));
///
///// Other charactors can be used for extracting.
///// 'a' extracts a single bit.
///assert_eq!(bitmatch!("1a10 1100", x), Some(0));
///assert_eq!(bitmatch!("10a0 1100", x), Some(1));
///
///// Multi-bit extracting by continuous charactors.
///assert_eq!(bitmatch!("1aaa a100", x), Some(5));
///
///// Multiple extracting.
///assert_eq!(bitmatch!("1aa0 aa00", x), Some((1, 3)));
///
///// If the extracting fields are adjacent, the different charactors can be used.
///assert_eq!(bitmatch!("1aab bccc", x), Some((1, 1, 4)));
///```
#[proc_macro]
pub fn bitmatch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();

    let mut input = input.into_iter();
    let pattern = input.next().expect("too less arguments");
    let comma = input.next().expect("too less arguments");
    let ident = input.next().expect("too less arguments");

    if let Some(_) = input.next() {
        panic!("too much arguments");
    }

    let pattern = match pattern {
        TokenTree::Literal(x) => x.to_string(),
        _ => {
            panic!("1st argument must be string literal");
        }
    };
    let pattern = if pattern.starts_with('\"') & pattern.ends_with('\"') {
        String::from(&pattern[1..pattern.len() - 1]).replace(" ", "")
    } else {
        panic!("1st argument must be string literal");
    };

    match comma {
        TokenTree::Punct(x) => {
            if x.as_char() != ',' {
                panic!("2nd argument must be ','");
            }
        }
        _ => {
            panic!("2nd argument must be ','");
        }
    }

    let ident = match &ident {
        TokenTree::Ident(x) => x,
        _ => {
            panic!("3rd argument must be identifier");
        }
    };

    match pattern.len() {
        1..=8 => gen_code_u8(pattern, ident),
        9..=16 => gen_code_u16(pattern, ident),
        17..=32 => gen_code_u32(pattern, ident),
        33..=64 => gen_code_u64(pattern, ident),
        65..=128 => gen_code_u128(pattern, ident),
        _ => {
            panic!("unsupported pattern length: {}", pattern.len());
        }
    }
}

macro_rules! gen_code {
    ($x:ty) => {
        paste::item! {
            fn [<gen_code_$x>](pattern: String, ident: &Ident) -> proc_macro::TokenStream {
                let mut bit_mask: $x = 0;
                let mut bit_pattern: $x = 0;

                let mut args_pos = Vec::new();
                let mut args_mask = Vec::new();

                let mut prev = None;
                let mut count = 0;

                for (i, bit) in pattern.chars().enumerate() {
                    bit_mask <<= 1;
                    bit_pattern <<= 1;
                    match bit {
                        '0' => {
                            bit_mask |= 1;
                            bit_pattern |= 0;
                        }
                        '1' => {
                            bit_mask |= 1;
                            bit_pattern |= 1;
                        }
                        '_' => {
                            bit_mask |= 0;
                            bit_pattern |= 0;
                        }
                        _ => {
                            bit_mask |= 0;
                            bit_pattern |= 0;
                        }
                    }
                    if let Some(x) = prev {
                        if x != bit && x != '0' && x != '1' && x != '_' {
                            let pos = (pattern.len() - i) as $x;
                            let mut mask: $x = 0;
                            for _ in 0..count {
                                mask <<= 1;
                                mask |= 1;
                            }
                            args_pos.push(pos);
                            args_mask.push(mask);

                            count = 0;
                        } else if x != bit {
                            count = 0;
                        }
                    }
                    count += 1;
                    prev = Some(bit);
                }
                if let Some(x) = prev {
                    if x != '0' && x != '1' && x != '_' {
                        let pos = 0 as $x;
                        let mut mask: $x = 0;
                        for _ in 0..count {
                            mask <<= 1;
                            mask |= 1;
                        }
                        args_pos.push(pos);
                        args_mask.push(mask);
                    }
                }

                let gen = quote! {
                    {
                        if #ident as $x & #bit_mask == #bit_pattern {
                            Some((
                                    #(
                                        (#ident as $x >> #args_pos) & #args_mask
                                    ),*
                                ))
                        } else {
                            None
                        }
                    }
                };

                gen.into()
            }
        }
    };
}

gen_code!(u8);
gen_code!(u16);
gen_code!(u32);
gen_code!(u64);
gen_code!(u128);
