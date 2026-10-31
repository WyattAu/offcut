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
- [ ] Transcription provider matrix in settings: local (default) / cloud
      (opt-in), per-task override
- [ ] Speaker diarization path for local (pyannote) or honest feature-absent
      labeling in the UI
- [ ] Kill the hard AssemblyAI key requirement at first-run

## R2: Rust render pipeline

- [ ] Render worker as a Rust job service (estate worker-kit + eventbus),
      ffmpeg via the existing CLI, driven by the same task queue tables
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
