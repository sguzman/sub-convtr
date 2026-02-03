# sub-convtr

Convert transcripts and subtitles between:

- SRT (.srt)
- WebVTT (.vtt)
- TXT (.txt)
- TSV (.tsv)
- JSON (.json)

This project takes the policy-driven scaffold described in `tmp/project.md` and applies it to a single CLI centered on a canonical `Transcript` model. The goal is to make it easy to normalize outputs from AI tools like Whisper, WhisperX, and ebook2audiobook into whichever subtitle format you need.

## Design summary

### Canonical model

All inputs become one in-memory representation:

- `Transcript { cues: Vec<Cue>, meta: Meta }`
- `Cue { start_ms, end_ms, text, speaker? }`

All outputs render from that shared model so every conversion is deterministic and testable.

### Parsing strategy

- SRT/VTT are parsed with `aspasia` (via `PlainSubtitle`).
- TXT, TSV, and JSON are parsed with custom adapters that honor the runtime policy in `config.toml`.

### Rendering strategy

- SRT/VTT writers are deterministic and respect wrap widths defined in the configuration.
- TXT/TSV/JSON writers also honor config-driven options such as time units and schema wrapping.

## Logging

We use `tracing` + `tracing-subscriber` for extensive diagnostics:

- Default log level is defined under `[logging].level` in `config.toml`.
- Override via `--log-level` or the `RUST_LOG` environment variable (which takes precedence).
- At `info` level you get format detection, cue counts, durations, and file decisions.
- At `debug` level you also get up to `logging.debug_cue_samples` cue samples with timings and lengths.

## Configuration

Policy knobs live in `config.toml`. Copy `config.example.toml` to `config.toml` to start customizing.

Key knobs:

- `policy.synthesize_timings`: synthesize timings for timestamp-free inputs.
- `policy.chars_per_second`: speaking-rate heuristic for synthesized durations.
- `policy.timestamp_offset_ms`: add an offset to all timestamps at export time.
- `formats.srt.wrap_width`, `formats.vtt.wrap_width`: control wrapping for those exports.
- `formats.tsv.time_units`: choose between `ms`, `seconds`, or `timestamp` for TSV exports.
- `formats.json.time_units` and `formats.json.wrapped`: control JSON time encoding and whether the output is wrapped in a schema envelope.

## CLI usage (examples)

Convert SRT -> JSON with logging:

```
subx convert input.srt --to json -o output.json
```

Convert VTT -> SRT:

```
subx convert input.vtt --to srt -o output.srt
```

Convert TSV -> VTT (forced input format):

```
subx convert input.tsv --from tsv --to vtt -o output.vtt
```

Pipe through stdin/stdout:

```
cat input.srt | subx convert - --from srt --to json --stdout
```

Write logs to a file (shell redirection):

```
subx convert input.srt --to json -o out.json 2> subx.log
```

## JSON shapes supported

### Wrapped schema (recommended)

When `formats.json.wrapped = true`, output looks like:

```
{
  "schema": "subxform.transcript",
  "version": 1,
  "cues": [
    { "start": 0.12, "end": 1.9, "text": "hello", "speaker": "SPEAKER_00" }
  ]
}
```

`start`/`end` are floats in seconds when `formats.json.time_units = "seconds"` or integers when set to `"ms"`.

### Bare array

When `wrapped=false`, the exporter emits:

```
[
  { "start": 0.12, "end": 1.9, "text": "hello" }
]
```

### Whisper-style JSON input

The parser already accepts structures like:

```
{ "segments": [ { "start": 0.12, "end": 1.9, "text": "hello" } ] }
```

Those segments are mapped into the canonical `Transcript` cues.

## Extending the converter

Add a new format by implementing `parse_<fmt>` (input -> `Transcript`) and `write_<fmt>` (model -> string), then wire the functions into `pipeline.rs` for parsing/rendering.

## License

CC0-1.0
