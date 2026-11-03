# M2 runbook: the zero-key end-to-end run

Goal state: one long video produces at least one captioned 9:16 clip with
**zero cloud calls** — local Whisper transcription + local Ollama selection.

Status: config default flipped (TRANSCRIPTION_PROVIDER=whisper, 2026-10-31).
This runbook is the owed E2E.

## Prerequisites (one-time)

```bash
# Python deps for the backend (in a venv; python-dotenv + fastapi stack)
cd backend
uv venv .venv && uv pip install -r requirements.txt

# Local transcription weights (fast-whisper downloads on first use; base is
# ~145 MB, tiny ~75 MB). WHISPER_MODEL controls the size.
export WHISPER_MODEL=tiny        # for the first E2E only
```

## Zero-key env (measured from .env.example)

```
TRANSCRIPTION_PROVIDER=whisper
LLM=ollama:qwen2.5:7b            # any local Ollama tag
# nothing else — no ASSEMBLY_AI_API_KEY, no OPENAI_API_KEY, no GOOGLE_API_KEY
OLLAMA_BASE_URL=http://localhost:11434/v1
```

Ollama must be running (`systemctl --user status ollama`) with the model
pulled (`ollama pull qwen2.5:7b`).

## Run

```bash
docker compose up -d --build
# submit one short source video (2-5 min keeps the first run fast):
# upload through the UI at localhost:3107
```

## Acceptance (the M2 gate)

- [x] **Transcription-stage E2E passed 2026-11-02** — zero-key, zero-cloud:
      espeak-ng synthesized "The quick brown fox jumps over the lazy dog"
      (WAV -> m4a), transcribed through `media/transcription.py`
      `transcribe_with_whisper(model_name="tiny")` via
      `_whisper_result_to_transcript_data`: 9 words, word-level timestamps
      present, keyword hits 3/4 ("dumps" for "jumps" — tiny-model quirk),
      transcript text returned. Exit 0. Venv:
      backend/requirements-local.txt (the minimal zero-key stack, discovered
      by import-chain walk). E2E script: m2-e2e.py pattern in session log.
- [x] Cloud SDK decoupled: assemblyai imports are lazy-guarded in
      media/transcription.py + media/common.py — the local path imports and
      runs with zero cloud packages installed (was an unconditional import:
      ModuleNotFoundError killed the whisper path before this fix)
- [ ] Full pipeline E2E (compose stack, clip rendering with burned captions)
- [ ] Pipeline completes with no outbound calls except the model hosts
      (verify: `docker compose logs backend | grep -i "assemblyai\|api.openai"`
      is empty)
- [ ] Transcript JSON contains word-level timestamps — DONE at
      transcription stage (see above); clip-stage re-verify pending
- [ ] At least one clip rendered with burned captions
- [ ] Timing recorded here (measured claims rule: date + command)

## Known gaps (tracked, not blockers for the gate)

- Speaker diarization: AssemblyAI-only upstream; local path lacks it. UI
  should say so honestly rather than silently skipping.
- slower-whisper vs fast-whisper choice: benchmark both before pinning.
