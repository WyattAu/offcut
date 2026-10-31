# offcut baseline: upstream build surface on CachyOS

Date: 2026-10-31. Seed commit: `4bc76f83b342` (upstream snapshot).

## Compose validation

```
docker compose config --quiet
```

Result: VALID (parses cleanly; no build attempted at baseline — see below).

## Environment surface (the fork thesis, measured)

`docker compose config` reports ~30 distinct unset variables across the
backend/worker services. Most belong to upstream's hosted-SaaS operation,
not to clipping:

- **Billing (drop for self-host)**: STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET,
  STRIPE_PRICE_ID, STRIPE_PRO_PRICE_ID, STRIPE_SCALE_PRICE_ID,
  STRIPE_PRO_ANNUAL_PRICE_ID, STRIPE_SCALE_ANNUAL_PRICE_ID,
  STRIPE_CHECKOUT_URL, STRIPE_CUSTOMER_PORTAL_URL, REVENUECAT_WEBHOOK_AUTH_HEADER,
  REVENUECAT_PRO_PRODUCT_IDS, REVENUECAT_SCALE_PRODUCT_IDS
- **Social publishing (opt-in later)**: YOUTUBE_OAUTH_CLIENT_ID/SECRET,
  TIKTOK_CLIENT_KEY/SECRET, INSTAGRAM_APP_ID/SECRET,
  SOCIAL_OAUTH_REDIRECT_BASE_URL, SOCIAL_PUBLIC_MEDIA_BASE_URL,
  YOUTUBE_DATA_API_KEY, APIFY_API_TOKEN
- **Ops noise**: DISCORD_FEEDBACK_WEBHOOK_URL, DISCORD_SALES_WEBHOOK_URL,
  AWS_REGION/AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY, REDIS_PASSWORD
- **Actually required for a local run**: ASSEMBLY_AI_API_KEY (upstream default
  transcription) + one LLM key (or `LLM=ollama:<model>` for zero cloud)

A self-hoster who wants local Whisper + local LLM needs **zero API keys**.
Today's compose makes that path undocumented and second-class. R1 fixes the
transcription half; R3 fixes selection.

## Required keys at first-run (upstream behavior, measured from .env.example)

- `ASSEMBLY_AI_API_KEY` — hard default for transcription
- `LLM` — defaults to `openrouter:anthropic/claude-sonnet-5.5` (cloud)
- `PEXELS_API_KEY` — optional, B-roll only

## Local-run plan (R1 target, measured acceptance)

`TRANSCRIPTION_PROVIDER=whisper` + `LLM=ollama:<model>` + no other keys →
one long video produces at least one captioned 9:16 clip without any cloud
call. Baseline for that benchmark lands with R1.
