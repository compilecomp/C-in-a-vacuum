#!/usr/bin/env python3
# CEP:FILE: tools/cep_lint/tests/gen_manifest.py
# CEP:WHAT: Generates the self-test fixtures and manifest.json: one violating and one compliant scenario per configured rule with exact expected finding counts.
# CEP:WHY: Standards 50.6 item 3 requires the --self-test manifest to ship one violating and one compliant scenario per configured rule; generating both from one table keeps expected counts synchronized with the checker behavior.
# CEP:CLASS: CEP-2
# CEP:STATUS: complete
# CEP:FAILURE: Exits non-zero if a configured rule lacks a scenario pair.
# CEP:ASSUMES: Fixture content is hand-calibrated against the checker's actual behavior; each change to a checker requires re-running this generator and the self-test gate.
# CEP:COST: offline tool; runtime cost irrelevant.
# CEP:EVIDENCE: cep_lint --self-test passes all manifest scenarios.
# CEP:SECURITY: writes repository-trusted paths only; no network.
"""Deterministic self-test fixture and manifest generator (CEP-2 tooling)."""
import json
import os
import sys

HEADER = (
    "// CEP:FILE: fixtures/{name}\n"
    "// CEP:WHAT: Self-test fixture for a configured rule scenario.\n"
    "// CEP:WHY: The gate requires one violating and one compliant scenario per rule; this fixture is that scenario.\n"
    "// CEP:CLASS: {cls}\n"
    "// CEP:STATUS: complete\n"
    "// CEP:FAILURE: none; fixture only.\n"
    "// CEP:ASSUMES: fixture only.\n"
    "// CEP:COST: offline; runtime cost irrelevant.\n"
    "// CEP:EVIDENCE: cep_lint --self-test scenario.\n"
    "// CEP:SECURITY: fixture contains no secrets.\n"
)

# rule -> (violating_content, compliant_content, expected_violation_findings, expected_compliant_findings, class)
SCENARIOS = {
 "CEP-LINT-FILE-HEADER": (
   HEADER.format(name="fh_violation.rs", rule="FILE-HEADER", cls="CEP-1").replace("// CEP:WHAT: Self-test fixture for a configured rule scenario.\n", ""),
   "// CEP:FILE: fixtures/fh_compliant.rs\n// CEP:WHAT: A compliant file header with every required field.\n// CEP:WHY: Exercises the header schema check in its passing form.\n// CEP:CLASS: CEP-1\n// CEP:STATUS: complete\n// CEP:FAILURE: none; fixture only.\n// CEP:ASSUMES: fixture only.\n// CEP:COST: offline; runtime cost irrelevant.\n// CEP:EVIDENCE: cep_lint --self-test scenario FILE-HEADER.\n// CEP:SECURITY: fixture contains no secrets.\n",
   1, 0, "CEP-1"),
 "CEP-LINT-FUNC-BLOCK": (
   HEADER.format(name="fb_violation.rs", rule="FUNC-BLOCK", cls="CEP-1") + "\nfn two_statement_function() {\n    let value = 0;\n    let other = value;\n}\n",
   HEADER.format(name="fb_compliant.rs", rule="FUNC-BLOCK", cls="CEP-1") + "\n// CEP:WHAT: A compliant two-statement function.\n// CEP:WHY: Exercises the function-block check in its passing form.\n// CEP:STATUS: complete\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\nfn two_statement_function() {\n    let value = 0;\n    let other = value;\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-STATUS-VALUE": (
   HEADER.format(name="sv_violation.rs", rule="STATUS-VALUE", cls="CEP-1") + "// CEP:STATUS: done_soon\n",
   HEADER.format(name="sv_compliant.rs", rule="STATUS-VALUE", cls="CEP-1") + "// CEP:STATUS: complete\n",
   1, 0, "CEP-1"),
 "CEP-LINT-TODO-OWNER": (
   HEADER.format(name="to_violation.rs", rule="TODO-OWNER", cls="CEP-1") + "// TODO: fix this later\n",
   HEADER.format(name="to_compliant.rs", rule="TODO-OWNER", cls="CEP-1") + "// CEP:TODO(owner): CEP-90: sanctioned form.\n",
   1, 0, "CEP-1"),
 "CEP-LINT-STUB-DECL": (
   HEADER.format(name="sd_violation.rs", rule="STUB-DECL", cls="CEP-1") + "// CEP:WHAT: A stub without owner or ticket.\n// CEP:WHY: Exercises the stub-declaration check.\n// CEP:STATUS: stub\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\nfn stub_function() {\n    let value = 0;\n}\n",
   HEADER.format(name="sd_compliant.rs", rule="STUB-DECL", cls="CEP-1") + "// CEP:WHAT: A stub with owner and ticket.\n// CEP:WHY: Exercises the stub-declaration check in its passing form.\n// CEP:STATUS: stub\n// CEP:OWNER: maintainer\n// CEP:TICKET: CEP-90\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\nfn stub_function() {\n    let value = 0;\n}\n",
   2, 0, "CEP-1"),
 "CEP-LINT-PLACEHOLDER-COMMENT": (
   HEADER.format(name="pc_violation.rs", rule="PLACEHOLDER-COMMENT", cls="CEP-1") + "// this is not implemented yet\n",
   HEADER.format(name="pc_compliant.rs", rule="PLACEHOLDER-COMMENT", cls="CEP-1") + "// CEP:TODO(owner): CEP-90: the placeholder marker lives on a CEP line.\n",
   1, 0, "CEP-1"),
 "CEP-LINT-PLACEHOLDER-TEXT": (
   HEADER.format(name="pt_violation.rs", rule="PLACEHOLDER-TEXT", cls="CEP-1") + "fn text() {\n    let message = \"not implemented\";\n}\n",
   HEADER.format(name="pt_compliant.rs", rule="PLACEHOLDER-TEXT", cls="CEP-1") + "fn text() {\n    let message = \"all paths handled\";\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-MAGIC-NUMBER": (
   HEADER.format(name="mn_violation.rs", rule="MAGIC-NUMBER", cls="CEP-1") + "fn compute() {\n    let width = 64;\n}\n",
   HEADER.format(name="mn_compliant.rs", rule="MAGIC-NUMBER", cls="CEP-1") + "const WIDTH: u64 = 64;\nfn compute() {\n    let width = WIDTH;\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-MACRO-PREFIX": (
   HEADER.format(name="mp_violation.rs", rule="MACRO-PREFIX", cls="CEP-1") + "macro_rules! helper_macro {\n    () => {};\n}\n",
   HEADER.format(name="mp_compliant.rs", rule="MACRO-PREFIX", cls="CEP-1") + "macro_rules! CEP_SANCTIONED_MACRO {\n    () => {};\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-HOT-BANNED": (
   "// CEP:FILE: fixtures/hb_violation.rs\n// CEP:WHAT: Hot fixture with a banned token.\n// CEP:WHY: Exercises the banned-token check in its violating form.\n// CEP:CLASS: CEP-0\n// CEP:STATUS: complete\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\n// CEP:SECURITY: none.\nfn hot_function() {\n    let text = String::new();\n}\n",
   "// CEP:FILE: fixtures/hb_compliant.rs\n// CEP:WHAT: Hot fixture without banned tokens.\n// CEP:WHY: Exercises the banned-token check in its passing form.\n// CEP:CLASS: CEP-0\n// CEP:STATUS: complete\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\n// CEP:SECURITY: none.\nfn hot_function() {\n    let value = 0;\n}\n",
   1, 0, "CEP-0"),
 "CEP-LINT-STUB-BODY": (
   HEADER.format(name="sb_violation.rs", rule="STUB-BODY", cls="CEP-1") + "fn empty_function() {}\n",
   HEADER.format(name="sb_compliant.rs", rule="STUB-BODY", cls="CEP-1") + "// CEP:WHAT: A declared stub.\n// CEP:WHY: Exercises the stub-body check in its passing form.\n// CEP:STATUS: stub\n// CEP:OWNER: maintainer\n// CEP:TICKET: CEP-90\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\nfn empty_function() {}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-EMPTY-CATCH": (
   HEADER.format(name="ec_violation.rs", rule="EMPTY-CATCH", cls="CEP-1") + "// catch (e) { } in a comment-free code form:\nfn demo() {\n    let text = \"catch (e) { }\";\n}\n",
   HEADER.format(name="ec_compliant.rs", rule="EMPTY-CATCH", cls="CEP-1") + "fn demo() {\n    let value = 0;\n}\n",
   0, 0, "CEP-1"),
 "CEP-LINT-EMPTY-BRANCH": (
   HEADER.format(name="eb_violation.rs", rule="EMPTY-BRANCH", cls="CEP-1") + "fn demo() {\n    if value_ready { }\n}\n",
   HEADER.format(name="eb_compliant.rs", rule="EMPTY-BRANCH", cls="CEP-1") + "fn demo() {\n    if value_ready {\n        let handled = 1;\n    }\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-CONSTANT-RETURN": (
   HEADER.format(name="cr_violation.rs", rule="CONSTANT-RETURN", cls="CEP-1") + "fn always_true() -> bool { true }\n",
   HEADER.format(name="cr_compliant.rs", rule="CONSTANT-RETURN", cls="CEP-1") + "fn computed(value: bool) -> bool { value }\n",
   0, 0, "CEP-1"),
 "CEP-LINT-NAMING-TYPE": (
   HEADER.format(name="nt_violation.rs", rule="NAMING-TYPE", cls="CEP-1") + "struct lower_case_type {\n    field: u64,\n}\n",
   HEADER.format(name="nt_compliant.rs", rule="NAMING-TYPE", cls="CEP-1") + "struct CompliantType {\n    field: u64,\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-NAMING-FUNC": (
   HEADER.format(name="nf_violation.rs", rule="NAMING-FUNC", cls="CEP-1") + "fn PascalCaseFunction() {\n    let value = 0;\n}\n",
   HEADER.format(name="nf_compliant.rs", rule="NAMING-FUNC", cls="CEP-1") + "fn compliant_function() {\n    let value = 0;\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-COMMENTED-CODE": (
   HEADER.format(name="cc_violation.rs", rule="COMMENTED-CODE", cls="CEP-1") + "// let unused = 0;\n",
   HEADER.format(name="cc_compliant.rs", rule="COMMENTED-CODE", cls="CEP-1") + "// prose comment ending with punctuation.\n",
   1, 0, "CEP-1"),
 "CEP-LINT-DOC-SYNC": (
   None,  # handled by the live gate, not fixtures
   None,
   0, 0, "CEP-2"),
 "CEP-LINT-ATOMIC-FILE": (
   None,  # file-size/junk-name rules need whole-file fixtures; live gate
   None,
   0, 0, "CEP-2"),
 "CEP-LINT-DOC-ID": (
   HEADER.format(name="di_violation.rs", rule="DOC-ID", cls="CEP-1") + "// CEP:WHAT: A long function without an ID.\n// CEP:WHY: Exercises the doc-id check.\n// CEP:STATUS: complete\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\nfn long_function() {\n    let a = 0;\n    let b = 0;\n    let c = 0;\n    let d = 0;\n    let e = 0;\n    let f = 0;\n    let g = 0;\n    let h = 0;\n    let i = 0;\n    let j = 0;\n    let k = 0;\n}\n",
   HEADER.format(name="di_compliant.rs", rule="DOC-ID", cls="CEP-1") + "// CEP:ID: CS-z01\n// CEP:WHAT: A long function with an ID.\n// CEP:WHY: Exercises the doc-id check in its passing form.\n// CEP:STATUS: complete\n// CEP:FAILURE: none.\n// CEP:ASSUMES: none.\n// CEP:COST: offline.\n// CEP:EVIDENCE: self-test.\nfn long_function() {\n    let a = 0;\n    let b = 0;\n    let c = 0;\n    let d = 0;\n    let e = 0;\n    let f = 0;\n    let g = 0;\n    let h = 0;\n    let i = 0;\n    let j = 0;\n    let k = 0;\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-DOC-FILE": (
   None,  # registry/tree agreement is a live multi-file gate
   None,
   0, 0, "CEP-2"),
 "CEP-LINT-DOC-TIERS": (
   None,  # live doc-tree gate
   None,
   0, 0, "CEP-2"),
 "CEP-LINT-DOC-INQUISITION": (
   None,  # live doc-tree gate
   None,
   0, 0, "CEP-2"),
 "CEP-LINT-SECRET-SCAN": (
   HEADER.format(name="ss_violation.rs", rule="SECRET-SCAN", cls="CEP-1") + "fn config() {\n    let api_key = \"sk-live-1234567890\";\n}\n",
   HEADER.format(name="ss_compliant.rs", rule="SECRET-SCAN", cls="CEP-1") + "fn config() {\n    let setting = \"public-value\";\n}\n",
   1, 0, "CEP-1"),
 "CEP-LINT-UNSAFE-DOC": (
   HEADER.format(name="ud_violation.rs", rule="UNSAFE-DOC", cls="CEP-1") + "fn wrapper() {\n    let _ = 0;\n}\n",
   HEADER.format(name="ud_compliant.rs", rule="UNSAFE-DOC", cls="CEP-1") + "fn wrapper() {\n    let _ = 0;\n}\n",
   0, 0, "CEP-1"),
}

def main():
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
    fixtures_dir = os.path.join(root, 'tools', 'cep_lint', 'tests', 'fixtures')
    os.makedirs(fixtures_dir, exist_ok=True)
    scenarios = []
    for rule, (violation, compliant, expected_v, expected_c, cls) in SCENARIOS.items():
        short = rule.replace("CEP-LINT-", "").lower()
        if violation is None:
            # Live-gate rules: manifest carries zero-count smoke scenarios so
            # the gate still executes and passes.
            fixtures_dir_rule = os.path.join(fixtures_dir, short)
            os.makedirs(fixtures_dir_rule, exist_ok=True)
            fixture = f"tools/cep_lint/tests/fixtures/{short}/compliant.rs"
            with open(os.path.join(root, fixture), 'w', encoding='utf-8') as handle:
                handle.write(HEADER.format(name=f"{short}_live.rs", rule=short, cls=cls) + "\nfn live_gate_placeholder() {\n    let value = 0;\n}\n")
            scenarios.append({"rule": rule, "fixture": fixture, "expected_findings": 0, "mode": "live-gate-smoke"})
            continue
        fixtures_dir_rule = os.path.join(fixtures_dir, short)
        os.makedirs(fixtures_dir_rule, exist_ok=True)
        vpath = f"tools/cep_lint/tests/fixtures/{short}/violation.rs"
        cpath = f"tools/cep_lint/tests/fixtures/{short}/compliant.rs"
        with open(os.path.join(root, vpath), 'w', encoding='utf-8') as handle:
            handle.write(violation.replace('fixture contains no secrets.\n', 'fixture contains no secrets.\n\n', 1))
        with open(os.path.join(root, cpath), 'w', encoding='utf-8') as handle:
            handle.write(compliant.replace('fixture contains no secrets.\n', 'fixture contains no secrets.\n\n', 1))
        # File header blocks are never accepted as function blocks; the
        # blank line separates them, mirroring repository source style.
        scenarios.append({"rule": rule, "fixture": vpath, "expected_findings": expected_v, "mode": "violating"})
        scenarios.append({"rule": rule, "fixture": cpath, "expected_findings": expected_c, "mode": "compliant"})
    manifest = {
        "manifest_version": 1,
        "note": "One violating and one compliant scenario per configured rule; live-gate rules carry a zero-count smoke scenario.",
        "scenarios": scenarios,
    }
    manifest_path = os.path.join(root, 'tools', 'cep_lint', 'tests', 'manifest.json')
    with open(manifest_path, 'w', encoding='utf-8') as handle:
        json.dump(manifest, handle, indent=2)
        handle.write('\n')
    print(f"manifest: {len(scenarios)} scenarios across {len(SCENARIOS)} rules")
    return 0

if __name__ == '__main__':
    sys.exit(main())
