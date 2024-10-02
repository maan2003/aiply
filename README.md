# Code Folding for LLM-Assisted Editing

## Overview

This program is designed to optimize the process of making code changes suggested by a Large Language Model (LLM). It addresses the challenge of efficiently handling large codebases when working with LLMs, which often have input size limitations and can be slower with larger inputs.

## Functionality

The program takes two inputs:
1. The original source code file
2. The suggested changes from an LLM in natural language format

It then produces a condensed version of the original file, keeping only the parts relevant to the suggested changes. This condensed version can be sent to another LLM for actual editing, resulting in faster processing and reduced latency.

## Running

```rust
cargo run --llm-output patch.md --source-file src/lib.rs
```
