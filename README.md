# offcut

One long video, a week of clips. AI finds the moments, cuts them vertical,
burns word-synced captions, and hands you post-ready files. Fully local where
possible, self-hosted by default.

offcut is a fork of [SupoClip](https://github.com/FujiwaraChoki/supoclip)
(AGPL-3.0), seeded from upstream commit `4bc76f83b342` (see
[docs/UPSTREAM.md](docs/UPSTREAM.md)).

## Why this fork exists

Upstream proves the pipeline (transcribe, select, score, render vertical clips
with word-synced captions). This fork removes the leash:

1. **Local transcription first.** Upstream defaults to a paid cloud ASR key.
   offcut defaults to local Whisper with word-level timestamps; cloud ASR
   stays as an opt-in quality tier.
2. **Rust render pipeline.** MoviePy becomes a Rust ffmpeg pipeline
   (WyattAu estate: worker-kit job orchestration, media-kit image handling,
   mp4-core container work). Same outputs, a fraction of the memory.
3. **Provider-agnostic selection.** Upstream's LLM selection already supports
   local Ollama; offcut makes the local path the documented default and adds
   estate model-router for cost-aware fallbacks.
4. **AGPL honored, not laundered.** This fork keeps AGPL-3.0 for everything it
   touches. That is the honest license for a self-hosted product.

## Status

Seeded, upstream-docker buildable. See [docs/ROADMAP.md](docs/ROADMAP.md).

## License

AGPL-3.0, preserved in [LICENSE](LICENSE). Upstream copyright FujiwaraChoki
and contributors; this fork's changes are (c) 2026 WyattAu under the same
AGPL-3.0. If you host offcut for others, the network clause applies to you
too. That is the point.

---

Fork additions and modifications: (c) 2026 WyattAu, AGPL-3.0. Upstream: see NOTICE.md and docs/UPSTREAM.md.
