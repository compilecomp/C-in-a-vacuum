# CEP-SMT-01
## Property-Directed Bounded SMT Verification Standard

**Version:** 1.0  
**Status:** Proposed Engineering Standard  
**Scope:** Systems programming, kernel infrastructure, allocators, compilers, runtimes, memory managers, and performance-critical code.

---

## 1. Objective

SMT solvers SHALL be used as property-directed verification engines to identify executions that violate explicitly defined engineering properties.

The objective is not to prove arbitrary program correctness through unconstrained exploration.

The objective is to:

1. Define a specific property to investigate.
2. Restrict the symbolic execution space to relevant behavior.
3. Establish explicit execution and solver resource limits.
4. Search for counterexamples.
5. Minimize discovered counterexamples.
6. Produce reproducible verification evidence.

Verification SHALL be driven by properties, not by indiscriminate source-code exploration.

## 2. Core principles

### 2.1 Property-first verification

Every verification task MUST specify a target property before execution.

Examples:

- No allocation returns the same frame twice.
- Failed operations do not mutate allocator state.
- Integer arithmetic cannot overflow.
- Memory accesses remain within valid bounds.
- Every allocated frame is eventually accounted for.
- A contiguous allocation never overlaps an existing allocation.

A verification task without an explicit property is invalid.

### 2.2 Bounded exploration

Every search MUST have explicit resource bounds.

The default initial limits are:

| Parameter | Default |
|---|---:|
| Maximum symbolic execution steps | 50 |
| Maximum path conditions | 50 |
| Maximum loop iterations | 10 |
| Maximum solver time | 30 seconds |
| Maximum solver memory | 512 MiB |
| Maximum counterexamples retained | 10 |

These are initial limits, not universal recommendations.

A search MAY increase limits when justified by the target property's complexity.

### 2.3 No false proofs

The following result classifications MUST remain distinct:

- SAT: A counterexample exists.
- UNSAT: No counterexample exists within the encoded model and bounds.
- UNKNOWN: The solver cannot determine satisfiability.
- TIMEOUT: The solver exceeded its time budget.
- BOUND_EXCEEDED: Exploration could not establish the property within configured bounds.
- TRANSLATION_FAILURE: The source program could not be faithfully represented.

UNKNOWN, TIMEOUT, BOUND_EXCEEDED, and TRANSLATION_FAILURE MUST NEVER be reported as successful verification.

## 3. Verification workflow

Every verification task SHALL follow this sequence.

### Step 1: Select the target

Identify:

- Source file.
- Function or method.
- Relevant dependencies.
- Input parameters.
- Mutable state.
- External assumptions.

Prefer function-level verification over whole-program verification when the property can be established locally.

### Step 2: Define the property

Express the property as a predicate over program inputs, outputs, and state transitions.

For example:

**Allocator transactionality**

Let:

- S₀ = allocator state before execution.
- S₁ = allocator state after execution.
- R = operation result.

Required property:

`R = Err => S₁ = S₀`

The state comparison MUST include every mutable field relevant to the contract, including:

- Bitmap contents.
- Allocation counter.
- Scan hint.
- Auxiliary metadata.

Checking only the externally visible allocation count is insufficient.

### Step 3: Define the input domain

Explicitly constrain symbolic inputs.

For example:

- `frame_count > 0`
- `scan_hint < frame_count`
- Bitmap capacity is sufficient.
- Initial allocation count matches bitmap population.
- Base frame arithmetic satisfies constructor invariants.

Input assumptions MUST be justified by actual program invariants or documented environmental contracts.

Do not introduce assumptions merely to eliminate inconvenient counterexamples.

### Step 4: Define the search budget

Each task MUST specify:

- Maximum execution steps.
- Maximum path conditions.
- Maximum loop iterations.
- Maximum solver time.
- Maximum solver memory.

The execution-step metric MUST be defined by the symbolic execution engine.

A source statement, intermediate-representation instruction, and symbolic transition are not equivalent units.

Path-condition count MUST be tracked independently from execution steps.

### Step 5: Prune the search space

Pruning MUST preserve all executions relevant to the selected property.

Permitted techniques include:

1. Input-domain restriction justified by invariants.
2. Function-level dependency reduction.
3. Irrelevant-state elimination.
4. Branch-condition simplification.
5. Constant propagation.
6. State abstraction.
7. Symmetry reduction.
8. Loop-bound restriction with explicit incomplete-search reporting.

Pruning MUST NOT silently remove paths that could violate the target property.

### Step 6: Generate symbolic constraints

Translate the selected execution into a logical representation.

Use:

- Bitvectors for fixed-width integer arithmetic.
- Arrays for indexed memory and bitmap state.
- Boolean expressions for branch conditions.
- Explicit state-transition relations for mutations.
- Uninterpreted functions only where their abstraction is justified.

Integer widths MUST match the source language's actual semantics.

For Rust, pay particular attention to checked arithmetic, wrapping arithmetic, indexing, and panic behavior.

### Step 7: Execute the solver

Ask the solver whether an execution exists that violates the selected property.

For a transactionality check:

`SAT(S₀ valid AND execution path AND R = Err AND S₁ != S₀)`

Interpretation:

- SAT: A violating execution exists.
- UNSAT: No violating execution exists within the encoded model and bounds.

### Step 8: Minimize counterexamples

Every SAT result SHOULD undergo counterexample minimization.

Attempt to reduce:

- Input values.
- Number of branches.
- Loop iterations.
- Memory state.
- Number of relevant conditions.
- Execution length.

The minimized witness MUST remain a valid counterexample.

### Step 9: Produce evidence

Every completed search MUST produce:

- Target function.
- Property identifier.
- Input assumptions.
- Search bounds.
- Solver and version.
- Encoding or translation version.
- Result classification.
- Counterexample, if applicable.
- Coverage limitations.
- Reproduction instructions.

## 4. Search profiles

Implement independently selectable verification profiles.

### PROFILE: TRANSACTIONALITY

Detect:

- Mutation before failure.
- Partial writes.
- Incomplete rollback.
- Incorrect error-path state.
- Accounting changes before validation.

Required predicate:

`Error(result) => state_after == state_before`

### PROFILE: ARITHMETIC

Detect:

- Integer overflow.
- Integer underflow.
- Invalid narrowing conversions.
- Incorrect signedness.
- Division by zero.
- Invalid address reconstruction.

Required predicate:

`ArithmeticViolation => SAT`

### PROFILE: BOUNDS

Detect:

- Out-of-range array accesses.
- Invalid bitmap indices.
- Invalid slice boundaries.
- Incorrect exclusive-end calculations.
- Invalid pointer offsets.

### PROFILE: OWNERSHIP

Detect:

- Duplicate allocation.
- Double free.
- Freeing unallocated resources.
- Overlapping reservations.
- Invalid ownership transitions.

### PROFILE: ACCOUNTING

Detect disagreement between independently maintained state.

Example:

`bitmap_popcount != allocated_count`

### PROFILE: CONTROL_FLOW

Detect:

- Unexpected unreachable states.
- Missing error propagation.
- Incorrect branch conditions.
- Invalid state transitions.
- Unexpected termination.

### PROFILE: COMPLEXITY

Search for executions exceeding specified symbolic operation budgets.

This profile MUST distinguish bounded execution cost from actual hardware execution time.

SMT-based path exploration alone does not establish cycle counts or cache behavior.

## 5. Allocator verification example

Target:

`FrameAllocator::alloc()`

Property:

Failed allocation MUST preserve the complete allocator state.

Initial assumptions:

- `frame_count > 0`
- `scan_hint < frame_count`
- Bitmap storage is valid.
- Initial accounting matches bitmap population.

Search configuration:

- Maximum steps: 50.
- Maximum path conditions: 50.
- Maximum loop iterations: 10.
- Solver timeout: 30 seconds.

Violation predicate:

`result == Err AND state_after != state_before`

Expected result:

UNSAT, provided the translation faithfully models the implementation and all relevant paths are covered within the bounds.

If SAT, retain the generated input and execution trace.

Repeat with separate properties for:

- Successful allocation uniqueness.
- Allocation exhaustion.
- Scan-hint correctness.
- Accounting preservation.

## 6. Solver selection

Recommended tools:

| Tool | Primary purpose |
|---|---|
| Kani | Rust verification |
| CBMC | C/C++ bounded model checking |
| Z3 | General SMT solving |
| cvc5 | Independent SMT solving |
| Boolector | Bitvector and array-heavy problems |

Use an appropriate program-analysis frontend rather than attempting to translate arbitrary source code directly into SMT formulas.

Independent solver runs MAY be used to cross-check results.

## 7. Performance requirements

The verification infrastructure MUST:

- Avoid unnecessary symbolic variables.
- Avoid unconstrained whole-program state.
- Prefer narrow function-level properties.
- Reuse proven environmental invariants.
- Track solver resource consumption.
- Record translation overhead separately from solving time.
- Preserve reproducibility.

Solver performance MUST NOT be confused with program performance.

A proof that an allocator uses at most 50 symbolic transitions does not establish that it executes within 50 CPU cycles.

## 8. Verification report format

Every search SHOULD produce the following report:

```text
CEP-SMT VERIFICATION REPORT

Target:
Property:
Profile:

Solver:
Solver Version:
Translation Engine:

Maximum Steps:
Maximum Path Conditions:
Maximum Loop Iterations:
Solver Timeout:
Memory Limit:

Result:
SAT / UNSAT / UNKNOWN / TIMEOUT /
BOUND_EXCEEDED / TRANSLATION_FAILURE

Explored Paths:
Pruned Paths:
Unexplored Paths:

Counterexample:
Minimized Counterexample:

Assumptions:
Limitations:

Reproduction Command:
```

## 9. Evidence classification

Use the following classifications:

**E0 - Unverified:** No formal search performed.

**E1 - Bounded Search:** A property was checked within explicit bounds.

**E2 - Bounded Verification:** All modeled paths within the stated bounds satisfy the property.

**E3 - Invariant Verification:** The property is established using verified state invariants and transition relations under documented assumptions.

**E4 - Unbounded Proof:** The property is proven for the entire specified mathematical domain.

A bounded UNSAT result MUST NOT be promoted to E4.

## 10. Final requirement

SMT verification exists to discover defects, establish explicit properties, and reduce uncertainty.

It MUST NOT become a mechanism for generating impressive-looking verification reports without corresponding proof obligations.

Every result must state exactly:

- What was checked.
- What was assumed.
- What was excluded.
- What was proven.
- What remains unknown.

**A smaller, precisely verified search space is preferable to a larger, poorly specified one.**
