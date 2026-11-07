# offcut ROADMAP

## R0: Baseline (this fork, week one)

- [ ] Upstream docker-compose up on the homelab; record the working .env in
      docs/BASELINE.md with dates (redact keys)
- [ ] License hygiene: AGPL-3.0 headers on every fork-added file; NOTICE with
      seed commit `4bc76f83b342`
- [ ] CI: frontend build + backend import smoke test on shared matrix

## R1: Local transcription default

- [ ] Local Whisper (faster-whisper) worker with word-level timestamps;
      parity target vs AssemblyAI on the upstream test set
- [x] Config default flipped (2026-11-01): TRANSCRIPTION_PROVIDER defaults
      to `whisper`; assemblyai becomes opt-in. Zero-key startup config
      verified by import. Full E2E (model download + one clip) still owed.
- [ ] Speaker diarization path for local (pyannote) or honest feature-absent
      labeling in the UI
- [ ] Kill the hard AssemblyAI key requirement at first-run

## R2: Rust render pipeline

- [x] Render worker seed: `rust/offcut-cut` — supervised spool worker on
      worker-kit 0.3 (JobSpec/Trigger/circuit breaker/drain), ffmpeg cut
      stage, CI acceptance test (synthetic video, duration asserted
      2.0s ± 0.2). Delivered 2026-11-01. Next: task-queue-tables
      integration + reframe/caption stages.
      Pipeline status: the zero-key full-stack E2E is BLOCKED at the
      selection stage — triple-confirmed model-capacity-bound on CPU
      (docs/M2-E2E-RUNBOOK.md, three models, three failure signatures).
      Unblocks on ROCm Ollama (AMD GPU) or a flash-tier selection key.
- [ ] Face-track reframing port: keep the fallback chain (MediaPipe → OpenCV
      DNN → Haar), benchmark each stage
- [ ] Caption burn via libass from the existing ASS generator; Ken Burns and
      hook-title stages ported with golden-frame tests
- [ ] MoviePy removed from the hot path; kept only for the built-in editor
      preview until parity is proven

## R3: Selection + cost

- [ ] Local-model selection prompt pack tuned for open models; cross-window
      rerank kept
- [ ] estate model-router for provider fallbacks with cost ceilings per task
- [ ] Manifest: cost per source-minute rendered in the task page, measured
      not estimated

## R4: Publishing + hardening

- [ ] Keep upstream MCP server; add REST auth (estate barbican/tokenkit
      patterns) and rate limits
- [ ] YouTube Shorts scheduling parity; TikTok/Instagram connectors behind
      feature flags
- [ ] Performance loop (views back into selection) as an opt-in sync job

---

(c) 2026 WyattAu, AGPL-3.0 (fork addition; upstream code remains AGPL-3.0 per LICENSE).
