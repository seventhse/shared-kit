// Segment Matcher – state-machine with char sets + escape support
// -------------------------------------------------------------
// * `*`  → any sequence (epsilon‑closure)
// * `?`  → single char (except '/')
// * `[abc]` positive char set, `[!a-z]` negative, supports ranges & escapes
// * `\\` escape – treats next char as literal
//   (e.g. `\*` matches literal '*')
// Algorithm: NFA active‑state vector (no recursion, no backtracking).
// -------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SegmentMatcher {
    tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Char(char), // literal
    AnyChar,    // ?
    AnySeq,     // *
    CharSet {
        // [abc] / [!a-z]
        positive: bool,
        bitset: [u8; 32], // 256‑bit ASCII table
    },
    Accept, // virtual EOF
}

impl SegmentMatcher {
    pub fn new(pattern: &str) -> Self {
        let mut tokens = Vec::new();
        let mut chars = pattern.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '*' => tokens.push(Token::AnySeq),
                '?' => tokens.push(Token::AnyChar),
                '[' => {
                    if let Some(set_token) = Self::parse_char_set(&mut chars) {
                        tokens.push(set_token);
                    } else {
                        // treat lone '[' as literal
                        tokens.push(Token::Char('['));
                    }
                }
                '\\' => {
                    // escape next char literally
                    if let Some(escaped) = chars.next() {
                        tokens.push(Token::Char(escaped));
                    } else {
                        tokens.push(Token::Char('\\'));
                    }
                }
                c => tokens.push(Token::Char(c)),
            }
        }
        tokens.push(Token::Accept);
        Self { tokens }
    }

    /// Parse inside [...]  returns None if not closed ']' found
    fn parse_char_set<I>(iter: &mut std::iter::Peekable<I>) -> Option<Token>
    where
        I: Iterator<Item = char>,
    {
        let mut positive = true;
        let mut bitset = [0u8; 32];
        let mut first = true;
        let mut prev: Option<char> = None;
        while let Some(&c) = iter.peek() {
            iter.next();
            // end of set
            if !first && c == ']' {
                return Some(Token::CharSet { positive, bitset });
            }
            if first {
                first = false;
                if c == '!' || c == '^' {
                    positive = false;
                    continue;
                }
            }
            if c == '\\' {
                // escaped char
                if let Some(esc) = iter.next() {
                    Self::set_bit(&mut bitset, esc as u8);
                    prev = Some(esc);
                    continue;
                }
            }
            if c == '-' && prev.is_some() {
                // range like a-z
                if let Some(&end) = iter.peek() {
                    if end != ']' {
                        iter.next();
                        let start = prev.unwrap() as u32;
                        let end_u = end as u32;
                        if start <= end_u && end_u < 128 {
                            for cp in start..=end_u {
                                Self::set_bit(&mut bitset, cp as u8);
                            }
                        }
                        prev = Some(end);
                        continue;
                    }
                }
            }
            Self::set_bit(&mut bitset, c as u8);
            prev = Some(c);
        }
        None // unclosed '[' ⇒ caller treats as literal
    }

    #[inline]
    fn set_bit(bitset: &mut [u8; 32], byte: u8) {
        let idx = (byte / 8) as usize;
        let mask = 1 << (byte % 8);
        bitset[idx] |= mask;
    }

    #[inline]
    fn bitset_contains(bitset: &[u8; 32], byte: u8) -> bool {
        let idx = (byte / 8) as usize;
        (bitset[idx] & (1 << (byte % 8))) != 0
    }

    /// ε‑closure: push idx and skip consecutive '*'
    fn epsilon_closure(&self, mut idx: usize, set: &mut Vec<usize>) {
        while idx < self.tokens.len() {
            set.push(idx);
            if self.tokens[idx] == Token::AnySeq {
                idx += 1; // * matches empty
            } else {
                break;
            }
        }
    }

    pub fn is_match(&self, segment: &str) -> bool {
        let mut curr: Vec<usize> = Vec::new();
        self.epsilon_closure(0, &mut curr);

        for ch in segment.chars() {
            let mut next: Vec<usize> = Vec::new();
            let b = ch as u8;
            for &state in &curr {
                match &self.tokens[state] {
                    Token::Char(c) if *c == ch => {
                        self.epsilon_closure(state + 1, &mut next);
                    }
                    Token::AnyChar => {
                        self.epsilon_closure(state + 1, &mut next);
                    }
                    Token::AnySeq => {
                        // stay or consume char
                        self.epsilon_closure(state, &mut next);
                    }
                    Token::CharSet { positive, bitset } => {
                        let contain = Self::bitset_contains(bitset, b);
                        if (*positive && contain) || (!positive && !contain) {
                            self.epsilon_closure(state + 1, &mut next);
                        }
                    }
                    _ => {}
                }
            }
            curr = next;
            if curr.is_empty() {
                return false;
            }
        }

        // accept if any active state reaches Accept via ε
        for &state in &curr {
            let mut idx = state;
            loop {
                match &self.tokens[idx] {
                    Token::AnySeq => idx += 1,
                    Token::Accept => return true,
                    _ => break,
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_set_basic() {
        let m = SegmentMatcher::new("[abc]x");
        assert!(m.is_match("ax"));
        assert!(m.is_match("bx"));
        assert!(!m.is_match("dx"));
    }

    #[test]
    fn char_set_neg() {
        let m = SegmentMatcher::new("[!0-9]*");
        assert!(m.is_match("hello"));
        assert!(!m.is_match("1foo"));
    }

    #[test]
    fn escape_and_range() {
        let m = SegmentMatcher::new("foo\\?bar");
        assert!(m.is_match("foo?bar"));
        assert!(!m.is_match("fooxbar"));

        let m2 = SegmentMatcher::new("[A-Z]file");
        assert!(m2.is_match("Zfile"));
        assert!(!m2.is_match("afile"));
    }
}
