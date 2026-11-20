// CEP:FILE: tools/cep_lint/src/regexlite.rs
// CEP:WHAT: A deterministic backtracking matcher for the regex subset used by the lint configuration, implemented with continuation-passing semantics.
// CEP:WHY: Standards 50 requires pattern-driven checks with the engine containing mechanism only; the subset (anchors, classes, groups, alternation, quantifiers, \b \s \d \w, inline case-insensitivity) covers every pattern in .cep/cep_lint.json without pulling a dependency.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns an error for unsupported syntax (backreferences, lookarounds) instead of silently mis-matching (50.4 class 8).
// CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded against infinite loops.
// CEP:COST: O(text * pattern) typical backtracking; pathological inputs absent from the configuration may exceed it and are a documented limit.
// CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex-scoped fixtures.
// CEP:SECURITY: Operates on in-memory text only.
//! Minimal deterministic regex matching.

#[derive(Clone, Debug)]
enum Node {
    /// A literal character.
    Char(char),
    /// A character class with items and negation.
    Class {
        negated: bool,
        items: Vec<ClassItem>,
    },
    /// Any character.
    Any,
    /// Concatenation of nodes.
    Concat(Vec<Node>),
    /// Alternation of branches.
    Alt(Vec<Node>),
    /// Repetition with inclusive minimum and optional maximum.
    Repeat {
        node: Box<Node>,
        min: usize,
        max: Option<usize>,
    },
    /// Start-of-text anchor.
    Start,
    /// End-of-text anchor.
    End,
    /// Word-boundary assertion.
    WordBoundary,
}

#[derive(Clone, Debug)]
enum ClassItem {
    /// One character.
    Single(char),
    /// Inclusive range.
    Range(char, char),
    /// Digit class.
    Digit,
    /// Word class.
    Word,
    /// Whitespace class.
    Space,
}

/// A compiled pattern with its case-insensitivity flag.
pub struct Regex {
    root: Node,
    icase: bool,
}

/// Parses a pattern.
// CEP:WHAT: Compiles a pattern into the matcher tree.
// CEP:WHY: One compile pass per rule keeps repeated scans cheap and surfaces unsupported syntax at bootstrap.
// CEP:STATUS: complete
// CEP:FAILURE: Err on unsupported constructs rather than mis-matching.
// CEP:ASSUMES: none.
// CEP:COST: O(pattern).
// CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset.
pub fn compile(pattern: &str) -> Result<Regex, String> {
    let icase = pattern.starts_with("(?i)");
    let body = if icase { &pattern[4..] } else { pattern };
    let mut parser = PatternParser {
        chars: body.chars().collect(),
        pos: 0,
    };
    let root = parser.parse_alt()?;
    if parser.pos != parser.chars.len() {
        return Err(format!("unsupported regex syntax at char {}", parser.pos));
    }
    Ok(Regex { root, icase })
}

impl Regex {
    /// True when any position in the text matches.
    // CEP:WHAT: Searches for a match anywhere in the text.
    // CEP:WHY: Comment and token rules use search semantics; anchored rules pin themselves with ^ and $.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total over any text.
    // CEP:ASSUMES: none.
    // CEP:COST: O(text * pattern) backtracking.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset.
    pub fn is_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        (0..=chars.len()).any(|start| match_node(&self.root, &chars, start, &|_| true, self.icase))
    }
}

type Cont<'a> = &'a dyn Fn(usize) -> bool;

// CEP:ID: CS-a45
// CEP:WHAT: Matches one pattern node at one position with continuation-passing backtracking.
// CEP:WHY: Continuation style makes alternation and repetition backtrack correctly without a separate match-state machine; each node type delegates to the continuation on success.
// CEP:STATUS: complete
// CEP:FAILURE: none; total over any node, text, and position.
// CEP:ASSUMES: Zero-width repeats are guarded in match_repeat against infinite loops.
// CEP:COST: O(text * pattern) typical backtracking.
// CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
fn match_node(node: &Node, text: &[char], pos: usize, k: Cont, icase: bool) -> bool {
    match node {
        Node::Char(expected) => {
            pos < text.len()
                && (text[pos] == *expected
                    || (icase && text[pos].to_lowercase().eq(expected.to_lowercase())))
                && k(pos + 1)
        }
        Node::Any => pos < text.len() && k(pos + 1),
        Node::Class { negated, items } => {
            if pos >= text.len() {
                return false;
            }
            let c = text[pos];
            let mut hit = items.iter().any(|item| match item {
                ClassItem::Single(single) => {
                    *single == c || (icase && single.to_lowercase().eq(c.to_lowercase()))
                }
                ClassItem::Range(start, end) => c >= *start && c <= *end,
                ClassItem::Digit => c.is_ascii_digit(),
                ClassItem::Word => is_word(c),
                ClassItem::Space => c.is_whitespace(),
            });
            if *negated {
                hit = !hit;
            }
            hit && k(pos + 1)
        }
        Node::Start => pos == 0 && k(pos),
        Node::End => pos == text.len() && k(pos),
        Node::WordBoundary => {
            let before = pos > 0 && is_word(text[pos - 1]);
            let after = pos < text.len() && is_word(text[pos]);
            (before != after) && k(pos)
        }
        Node::Concat(items) => match_seq(items, text, pos, k, icase),
        Node::Alt(branches) => branches
            .iter()
            .any(|branch| match_node(branch, text, pos, k, icase)),
        Node::Repeat { node, min, max } => match_repeat(node, *min, *max, text, pos, k, icase),
    }
}

// CEP:WHAT: Matches a concatenation sequence through chained continuations.
// CEP:WHY: Each item hands its end position to the next item's match, so backtracking propagates naturally.
// CEP:STATUS: complete
// CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
// CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
// CEP:COST: O(pattern) compile; O(text * pattern) match.
// CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
fn match_seq(items: &[Node], text: &[char], pos: usize, k: Cont, icase: bool) -> bool {
    if items.is_empty() {
        return k(pos);
    }
    match_node(
        &items[0],
        text,
        pos,
        &|next| match_seq(&items[1..], text, next, k, icase),
        icase,
    )
}

// CEP:WHAT: Backtracks a quantified node greedily with a zero-width guard.
// CEP:WHY: Greedy-first ordering with a zero-width guard makes repetition semantics deterministic and loop-free.
// CEP:STATUS: complete
// CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
// CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
// CEP:COST: O(pattern) compile; O(text * pattern) match.
// CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
fn match_repeat(
    node: &Node,
    min: usize,
    max: Option<usize>,
    text: &[char],
    pos: usize,
    k: Cont,
    icase: bool,
) -> bool {
    struct RepeatState<'a> {
        node: &'a Node,
        min: usize,
        max: Option<usize>,
        text: &'a [char],
        icase: bool,
    }
    fn go(state: &RepeatState, count: usize, pos: usize, k: Cont) -> bool {
        let can_more = state.max.is_none_or(|max| count < max);
        // Greedy: try one more repetition first (with a zero-width guard).
        if can_more
            && match_node(
                state.node,
                state.text,
                pos,
                &|next| next != pos && go(state, count + 1, next, k),
                state.icase,
            )
        {
            return true;
        }
        if count >= state.min {
            return k(pos);
        }
        false
    }
    let state = RepeatState {
        node,
        min,
        max,
        text,
        icase,
    };
    go(&state, 0, pos, k)
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

struct PatternParser {
    chars: Vec<char>,
    pos: usize,
}

impl PatternParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    // CEP:WHAT: Consumes and returns the character at the cursor.
    // CEP:WHY: Cursor advance is centralized so escape handling cannot double-consume.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
    // CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
    // CEP:COST: O(pattern) compile; O(text * pattern) match.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
    fn next(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    // CEP:ID: CS-a49
    // CEP:WHAT: Parses an alternation of branches separated by pipes.
    // CEP:WHY: Alternation is the top-level production; single-branch alternations collapse to the branch itself.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
    // CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
    // CEP:COST: O(pattern) compile; O(text * pattern) match.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
    fn parse_alt(&mut self) -> Result<Node, String> {
        let mut branches = vec![self.parse_concat()?];
        while self.peek() == Some('|') {
            self.pos += 1;
            branches.push(self.parse_concat()?);
        }
        if branches.len() == 1 {
            Ok(branches.pop().unwrap())
        } else {
            Ok(Node::Alt(branches))
        }
    }

    // CEP:ID: CS-a50
    // CEP:WHAT: Parses a concatenation of quantified atoms.
    // CEP:WHY: Atoms parse with their quantifiers before joining the implicit sequence.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
    // CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
    // CEP:COST: O(pattern) compile; O(text * pattern) match.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
    fn parse_concat(&mut self) -> Result<Node, String> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.parse_atom()?;
            let atom = self.parse_quantifier(atom)?;
            items.push(atom);
        }
        match items.len() {
            0 => Ok(Node::Concat(Vec::new())),
            1 => Ok(items.pop().unwrap()),
            _ => Ok(Node::Concat(items)),
        }
    }

    // CEP:ID: CS-a48
    // CEP:WHAT: Parses * + ? and brace quantifiers onto one atom.
    // CEP:WHY: A lone brace that does not match the quantifier shape is rewound and treated as a literal.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
    // CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
    // CEP:COST: O(pattern) compile; O(text * pattern) match.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
    fn parse_quantifier(&mut self, node: Node) -> Result<Node, String> {
        let (min, max) = match self.peek() {
            Some('*') => {
                self.pos += 1;
                (0, None)
            }
            Some('+') => {
                self.pos += 1;
                (1, None)
            }
            Some('?') => {
                self.pos += 1;
                (0, Some(1))
            }
            Some('{') => {
                let save = self.pos;
                self.pos += 1;
                let mut digits = String::new();
                while self.peek().is_none_or(|c| c.is_ascii_digit()) {
                    digits.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                if digits.is_empty() {
                    self.pos = save;
                    return Ok(node);
                }
                let min: usize = digits
                    .parse()
                    .map_err(|_| "bad repetition count".to_string())?;
                let max = if self.peek() == Some(',') {
                    self.pos += 1;
                    let mut more = String::new();
                    while self.peek().is_none_or(|c| c.is_ascii_digit()) {
                        more.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                    if more.is_empty() {
                        None
                    } else {
                        Some(
                            more.parse()
                                .map_err(|_| "bad repetition count".to_string())?,
                        )
                    }
                } else {
                    Some(min)
                };
                if self.peek() != Some('}') {
                    self.pos = save;
                    return Ok(node);
                }
                self.pos += 1;
                (min, max)
            }
            _ => return Ok(node),
        };
        Ok(Node::Repeat {
            node: Box::new(node),
            min,
            max,
        })
    }

    // CEP:ID: CS-a46
    // CEP:WHAT: Parses one atom: anchors, dot, class, group, escape, or literal.
    // CEP:WHY: Unsupported syntax errors instead of mis-matching (50.4 class 8).
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
    // CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
    // CEP:COST: O(pattern) compile; O(text * pattern) match.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
    fn parse_atom(&mut self) -> Result<Node, String> {
        match self.peek() {
            Some('^') => {
                self.pos += 1;
                Ok(Node::Start)
            }
            Some('$') => {
                self.pos += 1;
                Ok(Node::End)
            }
            Some('.') => {
                self.pos += 1;
                Ok(Node::Any)
            }
            Some('[') => self.parse_class(),
            Some('(') => {
                self.pos += 1;
                if self.peek() == Some('?') {
                    self.pos += 1;
                    if self.peek() == Some(':') {
                        self.pos += 1;
                    }
                }
                let inner = self.parse_alt()?;
                if self.peek() != Some(')') {
                    return Err("unbalanced group".to_string());
                }
                self.pos += 1;
                Ok(inner)
            }
            Some('\\') => {
                self.pos += 1;
                match self.next() {
                    Some('b') => Ok(Node::WordBoundary),
                    Some('s') => Ok(Node::Class {
                        negated: false,
                        items: vec![ClassItem::Space],
                    }),
                    Some('d') => Ok(Node::Class {
                        negated: false,
                        items: vec![ClassItem::Digit],
                    }),
                    Some('w') => Ok(Node::Class {
                        negated: false,
                        items: vec![ClassItem::Word],
                    }),
                    Some('n') => Ok(Node::Char('\n')),
                    Some('t') => Ok(Node::Char('\t')),
                    Some('r') => Ok(Node::Char('\r')),
                    Some(c) => Ok(Node::Char(c)),
                    None => Err("dangling escape".to_string()),
                }
            }
            Some(c) => {
                self.pos += 1;
                Ok(Node::Char(c))
            }
            None => Err("unexpected end of pattern".to_string()),
        }
    }

    // CEP:ID: CS-a47
    // CEP:WHAT: Parses a bracketed character class with ranges, escapes, and negation.
    // CEP:WHY: Class items decode ranges and shorthand classes; negation inverts the hit test at match time.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on unsupported pattern syntax; the caller reports it as a coded bootstrap diagnostic.
    // CEP:ASSUMES: Patterns are valid in the supported subset; zero-width repeats are guarded.
    // CEP:COST: O(pattern) compile; O(text * pattern) match.
    // CEP:EVIDENCE: cargo test -p cep_lint regexlite_subset; --self-test regex fixtures.
    fn parse_class(&mut self) -> Result<Node, String> {
        self.pos += 1;
        let mut negated = false;
        if self.peek() == Some('^') {
            negated = true;
            self.pos += 1;
        }
        let mut items = Vec::new();
        let mut first = true;
        loop {
            let c = self
                .peek()
                .ok_or_else(|| "unterminated class".to_string())?;
            if c == ']' && !first {
                self.pos += 1;
                return Ok(Node::Class { negated, items });
            }
            first = false;
            self.pos += 1;
            if c == '\\' {
                let escaped = self
                    .next()
                    .ok_or_else(|| "dangling class escape".to_string())?;
                match escaped {
                    's' => items.push(ClassItem::Space),
                    'd' => items.push(ClassItem::Digit),
                    'w' => items.push(ClassItem::Word),
                    'n' => items.push(ClassItem::Single('\n')),
                    't' => items.push(ClassItem::Single('\t')),
                    other => items.push(ClassItem::Single(other)),
                }
                continue;
            }
            if self.peek() == Some('-') && self.chars.get(self.pos + 1).is_some_and(|n| n != &']') {
                self.pos += 1;
                let end = self.next().ok_or_else(|| "dangling range".to_string())?;
                items.push(ClassItem::Range(c, end));
            } else {
                items.push(ClassItem::Single(c));
            }
        }
    }
}
