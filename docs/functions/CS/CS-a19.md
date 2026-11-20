# cep_lint check_file_header

Function ID: CS-a19 (domain CS)

## Tier 1: Beginner

Checks the file-header schema: position, required fields, and legal class.

## Tier 2: Silicon Reality

The header must be the first content, carry every required field, and declare an allowed conformance class; a missing header produces one finding at line 1 so the message anchors the file.

## Silent Failure Checklist

1. Fallbacks: none.
2. Truncation: none.
3. Wrapping: none.
4. Error codes: n/a; findings only.
