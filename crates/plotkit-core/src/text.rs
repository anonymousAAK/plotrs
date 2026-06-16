//! Inline text markup for super- and subscripts.
//!
//! Labels, titles, and annotations support a small, matplotlib-flavoured markup:
//!
//! - `^` raises the next character (or a `^{...}` group) into a **superscript**.
//! - `_` lowers the next character (or a `_{...}` group) into a **subscript**.
//! - `\^`, `\_`, and `\\` are escapes that emit a literal `^`, `_`, or `\`.
//!
//! The markup is resolved to Unicode super/subscript code points, so it renders
//! identically across every backend (PNG, SVG, PDF, WASM) with no special glyph
//! handling. Characters that have no Unicode super/subscript form are left as-is.
//!
//! ```
//! use plotkit_core::text::format_markup;
//!
//! assert_eq!(format_markup("x^2"), "x²");
//! assert_eq!(format_markup("H_2O"), "H₂O");
//! assert_eq!(format_markup("10^{-3}"), "10⁻³");
//! assert_eq!(format_markup("5 \\^ 2"), "5 ^ 2"); // escaped, left literal
//! ```

/// Returns the Unicode superscript form of `c`, if one exists.
fn superscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '\u{2070}',
        '1' => '\u{00B9}',
        '2' => '\u{00B2}',
        '3' => '\u{00B3}',
        '4' => '\u{2074}',
        '5' => '\u{2075}',
        '6' => '\u{2076}',
        '7' => '\u{2077}',
        '8' => '\u{2078}',
        '9' => '\u{2079}',
        '+' => '\u{207A}',
        '-' => '\u{207B}',
        '=' => '\u{207C}',
        '(' => '\u{207D}',
        ')' => '\u{207E}',
        'a' => 'ᵃ',
        'b' => 'ᵇ',
        'c' => 'ᶜ',
        'd' => 'ᵈ',
        'e' => 'ᵉ',
        'f' => 'ᶠ',
        'g' => 'ᵍ',
        'h' => 'ʰ',
        'i' => 'ⁱ',
        'j' => 'ʲ',
        'k' => 'ᵏ',
        'l' => 'ˡ',
        'm' => 'ᵐ',
        'n' => 'ⁿ',
        'o' => 'ᵒ',
        'p' => 'ᵖ',
        'r' => 'ʳ',
        's' => 'ˢ',
        't' => 'ᵗ',
        'u' => 'ᵘ',
        'v' => 'ᵛ',
        'w' => 'ʷ',
        'x' => 'ˣ',
        'y' => 'ʸ',
        'z' => 'ᶻ',
        _ => return None,
    })
}

/// Returns the Unicode subscript form of `c`, if one exists.
fn subscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '\u{2080}',
        '1' => '\u{2081}',
        '2' => '\u{2082}',
        '3' => '\u{2083}',
        '4' => '\u{2084}',
        '5' => '\u{2085}',
        '6' => '\u{2086}',
        '7' => '\u{2087}',
        '8' => '\u{2088}',
        '9' => '\u{2089}',
        '+' => '\u{208A}',
        '-' => '\u{208B}',
        '=' => '\u{208C}',
        '(' => '\u{208D}',
        ')' => '\u{208E}',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'h' => 'ₕ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'v' => 'ᵥ',
        'x' => 'ₓ',
        _ => return None,
    })
}

/// Resolves super/subscript markup in `input` to Unicode. See the [module
/// docs](self) for the syntax. Strings without `^`, `_`, or `\` are returned
/// unchanged (and the common case allocates only what it copies).
pub fn format_markup(input: &str) -> String {
    if !input.contains(['^', '_', '\\']) {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Escape: emit the next character literally (or a lone backslash).
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                } else {
                    out.push('\\');
                }
            }
            '^' => apply_markup(&mut out, &mut chars, superscript_char, '^'),
            '_' => apply_markup(&mut out, &mut chars, subscript_char, '_'),
            other => out.push(other),
        }
    }
    out
}

/// Handles a `^`/`_` marker: a braced `{...}` group or a single following char.
fn apply_markup(
    out: &mut String,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    map: fn(char) -> Option<char>,
    marker: char,
) {
    match chars.peek() {
        Some('{') => {
            chars.next(); // consume '{'
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                // Untranslatable characters are kept as-is inside the group.
                out.push(map(c).unwrap_or(c));
            }
        }
        Some(&next) => {
            if let Some(mapped) = map(next) {
                out.push(mapped);
                chars.next();
            } else {
                // Not markup (e.g. a stray "^" before a space): keep literal.
                out.push(marker);
            }
        }
        None => out.push(marker),
    }
}

#[cfg(test)]
mod tests {
    use super::format_markup;

    #[test]
    fn plain_text_unchanged() {
        assert_eq!(format_markup("hello world"), "hello world");
        assert_eq!(format_markup("temperature (C)"), "temperature (C)");
    }

    #[test]
    fn single_char_superscript() {
        assert_eq!(format_markup("x^2"), "x²");
        assert_eq!(format_markup("e^x"), "eˣ");
    }

    #[test]
    fn single_char_subscript() {
        assert_eq!(format_markup("H_2O"), "H₂O");
        assert_eq!(format_markup("x_i"), "xᵢ");
    }

    #[test]
    fn braced_groups() {
        assert_eq!(format_markup("10^{-3}"), "10⁻³");
        assert_eq!(format_markup("a_{ij}"), "a_{ij}".replace("_{ij}", "ᵢⱼ"));
        assert_eq!(format_markup("CO_{2}"), "CO₂");
    }

    #[test]
    fn escapes_are_literal() {
        assert_eq!(format_markup("5 \\^ 2"), "5 ^ 2");
        assert_eq!(format_markup("a\\_b"), "a_b");
        assert_eq!(format_markup("c:\\\\path"), "c:\\path");
    }

    #[test]
    fn stray_marker_before_space_kept() {
        assert_eq!(format_markup("2 ^ 3 is power"), "2 ^ 3 is power");
    }

    #[test]
    fn untranslatable_in_group_kept() {
        // 'Q' has no superscript form; it is preserved inside the group.
        assert_eq!(format_markup("x^{2Q}"), "x²Q");
    }

    #[test]
    fn trailing_marker() {
        assert_eq!(format_markup("value^"), "value^");
    }
}
