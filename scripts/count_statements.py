#!/usr/bin/env python3
# CEP:FILE: scripts/count_statements.py
# CEP:WHAT: Counts statements per Rust function with the linter-consistent definition (lines ending in ';' plus lines ending in '{') to enforce the standards 68.6.1 DOC-ID threshold mechanically.
# CEP:WHY: CEP-LINT-DOC-ID (standards 50.3) requires CEP:ID on functions with more than 10 statements; the counting definition must be identical to the linter's so review and enforcement never disagree (Law 8).
# CEP:CLASS: CEP-2
# CEP:STATUS: complete
# CEP:FAILURE: Exits non-zero if a listed file cannot be read.
# CEP:ASSUMES: Input files are UTF-8 Rust sources; raw string literals containing braces or semicolons can overcount, which only moves functions toward the threshold and never hides violations.
# CEP:COST: offline tool; runtime cost irrelevant.
# CEP:EVIDENCE: output reviewed against each module's CEP:ID declarations each run.
# CEP:SECURITY: reads repository-trusted files only; no network.
"""Deterministic statement counter matching cep_lint's DOC-ID definition.

Statement = a code line (comment stripped) ending in ';' or '{', inside the
function body. The definition deliberately overcounts (struct literals,
match-arm blocks) because for a >10 threshold, overcounting can only flag
early, never hide a violation.
"""
import re
import sys

FN = re.compile(r'^\s*(pub\s+)?(const\s+)?(unsafe\s+)?fn\s+([A-Za-z0-9_]+)')

def scan(path):
    lines = open(path, encoding='utf-8').read().split('\n')
    state = 'idle'
    name = None
    stmts = 0
    depth = 0
    out = []
    for raw in lines:
        code = raw.split('//')[0].strip()
        if state == 'idle':
            m = FN.match(raw)
            if m and not raw.strip().startswith('//'):
                name = m.group(4)
                stmts = 0
                if code.endswith('{'):
                    state = 'body'
                    depth = code.count('{') - code.count('}')
                else:
                    state = 'sig'
                    depth = 0
            continue
        if state == 'sig':
            if code.endswith('{') or '{' in code:
                depth += code.count('{') - code.count('}')
                if depth > 0:
                    state = 'body'
            continue
        opens = code.count('{')
        closes = code.count('}')
        depth += opens - closes
        if depth <= 0:
            out.append((name, stmts))
            state = 'idle'
            name = None
            stmts = 0
            depth = 0
            continue
        if not code:
            continue
        if code.endswith(';') or code.endswith('{'):
            stmts += 1
    return out

if __name__ == '__main__':
    exit_code = 0
    for path in sys.argv[1:]:
        try:
            for name, count in scan(path):
                marker = ' -> NEEDS CEP:ID' if count > 10 else ''
                if marker:
                    exit_code = 1
                print(f"{path}:{name}: {count}{marker}")
        except OSError as err:
            print(f"unreadable: {path}: {err}")
            exit_code = 2
    sys.exit(exit_code)
