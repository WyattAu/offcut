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

- [ ] Pipeline completes with no outbound calls except the model hosts
      (verify: `docker compose logs backend | grep -i "assemblyai\|api.openai"`
      is empty)
- [ ] Transcript JSON contains word-level timestamps
- [ ] At least one clip rendered with burned captions
- [ ] Timing recorded here (measured claims rule: date + command)

## Known gaps (tracked, not blockers for the gate)

- Speaker diarization: AssemblyAI-only upstream; local path lacks it. UI
  should say so honestly rather than silently skipping.
- slower-whisper vs fast-whisper choice: benchmark both before pinning.
