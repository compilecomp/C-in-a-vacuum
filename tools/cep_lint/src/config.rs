// CEP:FILE: tools/cep_lint/src/config.rs
// CEP:WHAT: Configuration loading: a minimal deterministic JSON parser and the typed lint configuration derived from it.
// CEP:WHY: Standards 50 requires the engine to be data-driven (every pattern, threshold, severity, and option lives in .cep/cep_lint.json); a hand-rolled parser keeps the tool dependency-free and hermetic (22.11 supply-chain minimization).
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: load returns a bootstrap error (exit 2 upstream) for unreadable files, parse failures with position, missing keys, and wrong types per the 50.4 bootstrap-code classes.
// CEP:ASSUMES: The parser accepts the JSON subset used by the repository configuration (objects, arrays, strings with escapes, integers, booleans, null); it rejects trailing content.
// CEP:COST: O(config size) parse; O(1) lookups on the typed struct.
// CEP:EVIDENCE: cargo test -p cep_lint config gates; --self-test scenarios.
// CEP:SECURITY: Reads repository-trusted files only; no network.
//! Minimal JSON parsing and typed configuration.

use std::collections::BTreeMap;

/// A parsed JSON value.
// CEP:WHAT: The parsed JSON value tree.
// CEP:WHY: Typed access to the data-driven configuration; the variant set covers the JSON subset the repository uses.
// CEP:STATUS: complete
// CEP:FAILURE: none; plain data type.
// CEP:ASSUMES: Integers are i64; objects preserve keys in a BTreeMap for determinism.
// CEP:COST: O(document size) to build.
// CEP:EVIDENCE: cargo test -p cep_lint config gates.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    /// JSON null.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// JSON integer.
    Int(i64),
    /// JSON string.
    Str(String),
    /// JSON array.
    Array(Vec<Json>),
    /// JSON object with ordered keys.
    Object(BTreeMap<String, Json>),
}

impl Json {
    /// Object member lookup.
    // CEP:WHAT: Looks up an object member by key.
    // CEP:WHY: The engine reads rules and options by name; missing keys surface as None and become bootstrap errors at the typed-load layer.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; returns None for absent keys and non-objects.
    // CEP:ASSUMES: none.
    // CEP:COST: O(log n) on the ordered map.
    // CEP:EVIDENCE: cargo test -p cep_lint config gates.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Array view.
    // CEP:WHAT: Returns the array contents.
    // CEP:WHY: Rule lists and fail_on severities are arrays.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; returns None for non-arrays.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p cep_lint config gates.
    pub fn as_array(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    /// String view.
    // CEP:WHAT: Returns the string contents.
    // CEP:WHY: Rule names and paths are strings.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; returns None for non-strings.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p cep_lint config gates.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(value) => Some(value),
            _ => None,
        }
    }

    /// Integer view.
    // CEP:WHAT: Returns the integer contents.
    // CEP:WHY: Severities and thresholds are integers.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; returns None for non-integers.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p cep_lint config gates.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Json::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// Boolean view.
    // CEP:WHAT: Returns the boolean contents.
    // CEP:WHY: icase flags are booleans.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; returns None for non-booleans.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p cep_lint config gates.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Object map view.
    // CEP:WHAT: Returns the object map for iterating members.
    // CEP:WHY: The rules table and file_topology limits are iterated member-wise; a direct map view avoids key enumeration.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; returns None for non-objects.
    // CEP:ASSUMES: none.
    // CEP:COST: O(1).
    // CEP:EVIDENCE: cargo test -p cep_lint config gates.
    pub fn get_object_ref(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Object(map) => Some(map),
            _ => None,
        }
    }

    // CEP:ID: CS-a26
    // CEP:WHAT: Parses a complete JSON document into the value tree, rejecting trailing content.
    // CEP:WHY: The data-driven engine needs typed access to rules; rejecting trailing content catches truncated configurations instead of silently accepting them (50.4 class 2).
    // CEP:STATUS: complete
    // CEP:FAILURE: Returns a descriptive error with byte offset for malformed documents.
    // CEP:ASSUMES: Input is UTF-8; escapes are the JSON standard set.
    // CEP:COST: O(size) single pass; allocation proportional to document size.
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    pub fn parse(text: &str) -> Result<Json, String> {
        let bytes = text.as_bytes();
        let mut parser = Parser { bytes, pos: 0 };
        parser.skip_ws();
        let value = parser.parse_value()?;
        parser.skip_ws();
        if parser.pos != bytes.len() {
            return Err(format!("trailing content at byte {}", parser.pos));
        }
        Ok(value)
    }
}

/// The recursive-descent JSON parser state.
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    // CEP:WHAT: Skips whitespace between JSON tokens.
    // CEP:WHY: JSON allows whitespace between tokens; skipping keeps the value parser position-agnostic.
    // CEP:STATUS: complete
    // CEP:FAILURE: none; total.
    // CEP:ASSUMES: none.
    // CEP:COST: O(whitespace).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len()
            && matches!(self.bytes[self.pos], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.pos += 1;
        }
    }

    // CEP:ID: CS-a27
    // CEP:WHAT: Parses one JSON value by dispatching on the first byte.
    // CEP:WHY: The recursive-descent core; each production delegates to its dedicated parser so errors carry positions.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err with a byte position for malformed values.
    // CEP:ASSUMES: none.
    // CEP:COST: O(size of the value).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.bytes.get(self.pos) {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(Json::Str(self.parse_string()?)),
            Some(b't') => self.parse_literal("true", Json::Bool(true)),
            Some(b'f') => self.parse_literal("false", Json::Bool(false)),
            Some(b'n') => self.parse_literal("null", Json::Null),
            Some(_) => self.parse_number(),
            None => Err("unexpected end of document".to_string()),
        }
    }

    // CEP:WHAT: Parses true, false, or null literals.
    // CEP:WHY: Literal parsing must consume exactly the literal text so trailing checks stay valid.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err when the text does not match the literal.
    // CEP:ASSUMES: none.
    // CEP:COST: O(literal length).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn parse_literal(&mut self, text: &str, value: Json) -> Result<Json, String> {
        if self.bytes[self.pos..].starts_with(text.as_bytes()) {
            self.pos += text.len();
            Ok(value)
        } else {
            Err(format!("invalid literal at byte {}", self.pos))
        }
    }

    // CEP:WHAT: Parses an integer literal.
    // CEP:WHY: The configuration uses integers only; out-of-range numbers fail loudly rather than wrapping.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err for out-of-range or malformed numbers.
    // CEP:ASSUMES: none.
    // CEP:COST: O(digits).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn parse_number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.bytes.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "invalid number encoding".to_string())?;
        text.parse::<i64>()
            .map(Json::Int)
            .map_err(|_| format!("number out of integer range at byte {start}"))
    }

    // CEP:ID: CS-a28
    // CEP:WHAT: Parses a JSON string with escape handling and UTF-8 validation.
    // CEP:WHY: Patterns and paths arrive as strings; escape sequences decode byte-exactly and UTF-8 is validated once at the closing quote.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err for unterminated strings, unknown escapes, invalid code points, and invalid UTF-8.
    // CEP:ASSUMES: none.
    // CEP:COST: O(string length).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn parse_string(&mut self) -> Result<String, String> {
        if self.bytes.get(self.pos) != Some(&b'"') {
            return Err(format!("expected string at byte {}", self.pos));
        }
        self.pos += 1;
        let mut raw: Vec<u8> = Vec::new();
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'"' => {
                    self.pos += 1;
                    return String::from_utf8(raw)
                        .map_err(|_| "invalid UTF-8 in string".to_string());
                }
                b'\\' => {
                    self.pos += 1;
                    let escaped = *self
                        .bytes
                        .get(self.pos)
                        .ok_or_else(|| "unterminated escape".to_string())?;
                    self.pos += 1;
                    match escaped {
                        b'"' => raw.push(b'"'),
                        b'\\' => raw.push(b'\\'),
                        b'/' => raw.push(b'/'),
                        b'n' => raw.push(b'\n'),
                        b't' => raw.push(b'\t'),
                        b'r' => raw.push(b'\r'),
                        b'b' => raw.push(8),
                        b'f' => raw.push(12),
                        b'u' => {
                            let hex = self
                                .bytes
                                .get(self.pos..self.pos + 4)
                                .ok_or_else(|| "truncated unicode escape".to_string())?;
                            let code = u32::from_str_radix(
                                std::str::from_utf8(hex)
                                    .map_err(|_| "invalid unicode escape".to_string())?,
                                16,
                            )
                            .map_err(|_| "invalid unicode escape".to_string())?;
                            self.pos += 4;
                            let mut buffer = [0u8; 4];
                            let character = char::from_u32(code)
                                .ok_or_else(|| "invalid code point".to_string())?;
                            raw.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
                        }
                        _ => return Err(format!("unknown escape at byte {}", self.pos)),
                    }
                }
                _ => {
                    let byte = self.bytes[self.pos];
                    self.pos += 1;
                    // UTF-8 continuation bytes accumulate raw and are
                    // validated once at the closing quote.
                    raw.push(byte);
                }
            }
        }
        Err("unterminated string".to_string())
    }

    // CEP:ID: CS-a29
    // CEP:WHAT: Parses a JSON array.
    // CEP:WHY: Array productions keep comma structure explicit so truncation is caught.
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on missing separators or terminators.
    // CEP:ASSUMES: none.
    // CEP:COST: O(elements).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn parse_array(&mut self) -> Result<Json, String> {
        self.pos += 1;
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.bytes.get(self.pos) == Some(&b']') {
                self.pos += 1;
                return Ok(Json::Array(items));
            }
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bytes.get(self.pos) {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(format!("expected , or ] at byte {}", self.pos)),
            }
        }
    }

    // CEP:ID: CS-a30
    // CEP:WHAT: Parses a JSON object into the ordered key map.
    // CEP:WHY: Rules and options are objects; ordered keys keep iteration deterministic (6.5).
    // CEP:STATUS: complete
    // CEP:FAILURE: Err on missing separators, colons, or terminators.
    // CEP:ASSUMES: Duplicate keys are last-wins, matching common tooling.
    // CEP:COST: O(members).
    // CEP:EVIDENCE: --self-test config-scoped scenarios.
    fn parse_object(&mut self) -> Result<Json, String> {
        self.pos += 1;
        let mut map = BTreeMap::new();
        loop {
            self.skip_ws();
            if self.bytes.get(self.pos) == Some(&b'}') {
                self.pos += 1;
                return Ok(Json::Object(map));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bytes.get(self.pos) != Some(&b':') {
                return Err(format!("expected : at byte {}", self.pos));
            }
            self.pos += 1;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_ws();
            match self.bytes.get(self.pos) {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Object(map));
                }
                _ => return Err(format!("expected , or }} at byte {}", self.pos)),
            }
        }
    }
}

/// The typed lint configuration the engine consumes.
// CEP:WHAT: The typed lint configuration with fail_on severities, globs, and the rule table.
// CEP:WHY: One typed struct validates the schema once so check primitives can consume options without re-validating.
// CEP:STATUS: complete
// CEP:FAILURE: load returns coded bootstrap errors per 50.4.
// CEP:ASSUMES: The schema is the one documented in standards 50.3; unknown top-level keys are ignored forward-compatibly.
// CEP:COST: O(config size).
// CEP:EVIDENCE: --self-test config-scoped scenarios.
pub struct LintConfig {
    /// Severities that force exit 1.
    pub fail_on: Vec<i64>,
    /// File globs considered first-party (suffix match).
    pub file_globs: Vec<String>,
    /// Rule name -> rule body.
    pub rules: BTreeMap<String, Rule>,
    /// DOC-SYNC document path.
    pub doc_path: String,
    /// DOC-SYNC rule heading pattern.
    pub doc_heading_pattern: String,
    /// DOC-SYNC version line pattern.
    pub doc_version_pattern: String,
    /// Expected standard version for DOC-SYNC.
    pub standard_version: String,
}

/// One configured rule with its parameters.
// CEP:WHAT: One configured rule with severity, check primitive, mode, and options.
// CEP:WHY: The engine dispatches on check and mode while options carry the data; one struct keeps the dispatch uniform.
// CEP:STATUS: complete
// CEP:FAILURE: none; plain data type.
// CEP:ASSUMES: Severity is validated at load time (0..=3).
// CEP:COST: O(1) per field access.
// CEP:EVIDENCE: --self-test config-scoped scenarios.
pub struct Rule {
    /// 0..3 severity per the 34.2 classes.
    pub severity: i64,
    /// Check primitive name.
    pub check: String,
    /// Optional mode within the primitive.
    pub mode: String,
    /// Raw option map for the primitive.
    pub options: BTreeMap<String, Json>,
}

/// Loads and validates the configuration.
// CEP:ID: CS-a31
// CEP:WHAT: Loads the lint configuration from disk and validates the schema.
// CEP:WHY: Bootstrap validation (missing keys, wrong types, severity range) must fail with coded errors before any scan, per the 50.4 bootstrap classes.
// CEP:STATUS: complete
// CEP:FAILURE: Returns Err with a class-tagged message for unreadable files (class 1), parse failures (class 2), missing keys (class 4), wrong types (class 5), and out-of-range severities (class 7).
// CEP:ASSUMES: The configuration schema is the one documented in standards 50.3.
// CEP:COST: O(config size) parse plus O(rules) validation.
// CEP:EVIDENCE: --self-test config-scoped scenarios.
pub fn load(path: &str) -> Result<(LintConfig, Json), String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|err| format!("CEP-LINT-BOOT-1: unreadable configuration {path}: {err}"))?;
    let document =
        Json::parse(&raw).map_err(|err| format!("CEP-LINT-BOOT-2: configuration parse: {err}"))?;
    let fail_on = document
        .get("fail_on")
        .and_then(Json::as_array)
        .ok_or("CEP-LINT-BOOT-4: missing fail_on")?
        .iter()
        .map(|value| {
            value
                .as_int()
                .ok_or_else(|| "CEP-LINT-BOOT-5: fail_on must be integers".to_string())
        })
        .collect::<Result<Vec<i64>, String>>()?;
    for severity in &fail_on {
        if !(0..=3).contains(severity) {
            return Err(format!(
                "CEP-LINT-BOOT-7: severity {severity} outside 0..=3"
            ));
        }
    }
    let file_globs = document
        .get("file_globs")
        .and_then(Json::as_array)
        .ok_or("CEP-LINT-BOOT-4: missing file_globs")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|text| text.to_string())
                .ok_or_else(|| "CEP-LINT-BOOT-5: file_globs must be strings".to_string())
        })
        .collect::<Result<Vec<String>, String>>()?;
    let rules_json = document
        .get("rules")
        .and_then(Json::get_object_ref)
        .ok_or("CEP-LINT-BOOT-4: missing rules")?;
    let mut rules = BTreeMap::new();
    for (name, body) in rules_json {
        let severity = body
            .get("severity")
            .and_then(Json::as_int)
            .ok_or_else(|| format!("CEP-LINT-BOOT-4: rule {name} missing severity"))?;
        if !(0..=3).contains(&severity) {
            return Err(format!(
                "CEP-LINT-BOOT-7: rule {name} severity out of range"
            ));
        }
        let check = body
            .get("check")
            .and_then(Json::as_str)
            .ok_or_else(|| format!("CEP-LINT-BOOT-4: rule {name} missing check"))?
            .to_string();
        let mode = body
            .get("mode")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string();
        let mut options = BTreeMap::new();
        if let Some(map) = body.get_object_ref() {
            for (key, value) in map {
                if key != "severity" && key != "check" && key != "mode" {
                    options.insert(key.clone(), value.clone());
                }
            }
        }
        rules.insert(
            name.clone(),
            Rule {
                severity,
                check,
                mode,
                options,
            },
        );
    }
    let doc_sync = document
        .get("doc_sync")
        .ok_or("CEP-LINT-BOOT-4: missing doc_sync")?;
    let standard_version = document
        .get("standard.version")
        .and_then(Json::as_str)
        .ok_or("CEP-LINT-BOOT-4: missing standard.version")?
        .to_string();
    let config = LintConfig {
        fail_on,
        file_globs,
        rules,
        doc_path: doc_sync
            .get("doc_path")
            .and_then(Json::as_str)
            .ok_or("CEP-LINT-BOOT-4: missing doc_sync.doc_path")?
            .to_string(),
        doc_heading_pattern: doc_sync
            .get("heading_pattern")
            .and_then(Json::as_str)
            .ok_or("CEP-LINT-BOOT-4: missing doc_sync.heading_pattern")?
            .to_string(),
        doc_version_pattern: doc_sync
            .get("version_pattern")
            .and_then(Json::as_str)
            .ok_or("CEP-LINT-BOOT-4: missing doc_sync.version_pattern")?
            .to_string(),
        standard_version,
    };
    Ok((config, document))
}
