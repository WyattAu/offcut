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

## Full-stack attempt (2026-11-03, compose stack, measured)

| Stage | Result |
|---|---|
| Upload (m4a -> upload:// ref) | OK (FK fix: e2e user row seeded via psql) |
| Local Whisper in-container | OK — model medium, CPU/FP32, 2.2 min for 5 s audio; 9 words cached (exactly the 9-word phrase) |
| LLM selection (tinyllama via host Ollama) | FAILED — capacity, not configuration |

Model-sizing finding (measured): tinyllama on CPU answers a 30-token probe
in 41 s. The transcript-analysis prompt is orders larger -> exceeds the
attempt window. **Options, in order of practicality:** (1) GPU Ollama (the
box's AMD GPU needs the ROCm variant of Ollama — user decision), (2) a
bigger/faster model is still CPU-bound, (3) bump the LLM attempt timeout and
accept a long analysis, (4) API fallback for the selection stage only.

## Local-stack env gotchas (recorded)

- Compose passes only explicitly-listed env vars: ALLOW_UNSIGNED_BACKEND_AUTH
  must be added to the override's `environment:` (it is, in our override).
- Unsigned-auth requests need header `x-supoclip-user-id: <user>` AND a
  seeded users row (FK from media_upload_sessions): seeded via psql
  (`INSERT INTO users (id,name,email,"emailVerified") VALUES ('e2e-local',...)`).
- Host-port conflicts with estate infra (5432/6379/8000): resolved by
  docker-compose.override.yml (backend 18000, frontend 13107, redis/postgres
  unmapped) + `docker network prune` (address pools exhausted at 33 networks).
- Worker scans `out/build/src` only for ggml backend modules — the
  variant-dispatch CPU modules land in `out/build/bin/` (symlinked).

## Final measured finding (2026-11-03): the selection stage is model-capacity-bound

Attempt 2 with qwen2.5:0.5b (fastest CPU model, 1800s budget): the pipeline
reached the LLM stage and **failed with "Exceeded maximum output retries (2)"**
— pydantic-ai's schema-repair retries exhausted. The 0.5B model responds but
cannot reliably emit the structured analysis schema. This is a capability
ceiling, not configuration: the failure signature is stable across
tinyllama (timeout) and qwen2.5:0.5b (schema retries).

### The three viable paths (user decision required)

1. **GPU inference** — the box's AMD GPU + ROCm Ollama variant: ~10-30x
   throughput, makes even 7-8B models viable. Requires the ROCm Ollama
   install (user action).
2. **llama3.2:3b on CPU + 3600s budget** — 2-3x slower than 0.5b per token
   but materially better schema compliance; still may exceed retries.
   Requires: bigger budget + llama3.2:3b pull (~2 GB).
3. **One cheap API key for the selection stage only** — Gemini free tier or
   a flash-tier model: the heavy paths (upload, transcription, storage,
   render) stay zero-key/local; one small call per task for the "brain".
   Honest framing: the zero-key promise covers the pipeline's compute, the
   selection call is the one metered exception.

### What IS proven zero-key (2026-11-02, unchanged)

Upload -> local Whisper (word timestamps) -> transcript cache -> LLM-stage
entry. The transcription stage — the heavy compute — runs fully local.
