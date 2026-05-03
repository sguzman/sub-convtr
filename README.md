# sub-convtr

`sub-convtr` converts transcripts and subtitles between multiple text, subtitle, and JSON formats through a canonical transcript model.

## Intent

Normalize the growing mess of subtitle/transcript outputs from AI and media tools so conversion between formats is deterministic and policy-driven.

## Ambition

The design docs and canonical-model emphasis show an ambition to be a dependable format-normalization layer rather than a collection of pairwise converters.

## Current Status

The CLI, config, model, and pipeline modules are already present, along with examples and input-shape documentation. It appears usable for real conversion tasks.

## Core Capabilities Or Focus Areas

- Convert among SRT, VTT, ASS/SSA, TXT, TSV, and JSON shapes.
- Use a canonical transcript model internally.
- Apply config-driven conversion policies.
- Handle AI-oriented transcript JSON variants.
- Support examples for common conversion paths.

## Project Layout

- `examples/`: sample inputs, example configs, or demonstration workflows.
- `src/`: Rust source for the main crate or application entrypoint.
- `Cargo.toml`: crate or workspace manifest and the first place to check for package structure.

## Setup And Requirements

- Rust toolchain.
- Subtitle/transcript inputs in one of the supported formats.
- Optional config files for repeatable policy.

## Build / Run / Test Commands

```bash
cargo build
cargo test
cargo run -- --help
```

## Notes, Limitations, Or Known Gaps

- Format edge cases matter more than raw size here, so sample coverage is important.
- The canonical model is the main design commitment of the project.

## Next Steps Or Roadmap Hints

- Keep the canonical transcript model stable as more formats or AI outputs are added.
- Add more fixtures for lossy or ambiguous conversion boundaries.
