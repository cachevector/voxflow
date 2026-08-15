---
title: Provider keys
description: Set up the AI rewrite pass with Groq, OpenAI, or your own self-hosted OpenAI-compatible server — and how VoxFlow stores those keys.
group: Using VoxFlow
order: 5
---

Transcription is local, so it needs no key at all. The one step that can leave your machine
is the **AI rewrite pass**, which turns a raw transcript into a clean sentence.

VoxFlow talks to that step through a single generic OpenAI-compatible HTTP adapter. Any
service or server that speaks that API will work — you supply a base URL and, if it needs
one, a key.

## Where keys are stored

Keys go into the **macOS Keychain** through the `keyring` crate, under a named secret
reference. They are never written to `settings.json`.

You can inspect or remove them yourself in Keychain Access, or from the command line with the
`voxflowctl` key management commands in the repository.

## Groq

The default for the rewrite pass, because it is fast and the cleanup call is small.

1. Create a key at [console.groq.com](https://console.groq.com).
2. Open **Settings → AI Cleanup (Groq)** and paste it in.

Local Whisper transcription does not use Groq — only the cleanup call does.

## OpenAI

Same adapter, different base URL and key. Create a key at
[platform.openai.com](https://platform.openai.com) and enter it under the provider settings.

OpenAI can also handle transcription if you would rather not run Whisper locally, though that
gives up the on-device privacy guarantee.

## A server you run yourself

Point VoxFlow at any OpenAI-compatible server on your own hardware — `llama.cpp-server` or
Ollama in OpenAI-compatible mode for the rewrite, `whisper.cpp-server` for transcription if
you want that remote too.

1. Set the provider kind to **Custom Endpoint**.
2. Enter the base URL, for example `http://homeserver.tailnet-name.ts.net:8080/v1`.
3. Enter a key only if your server requires one.

Marginal cost is effectively zero — you are paying for hardware you already own. Reaching
that server away from home is up to you; a VPN or tunnel like Tailscale is the usual answer.
VoxFlow simply calls whatever base URL it is given.

## No rewrite at all

Turn the rewrite pass off and VoxFlow makes no network calls whatsoever. You get the raw
Whisper transcript, fillers and all, entirely on device.

## What it costs

The rewrite pass runs over a short piece of text, so it is cheap on any provider. If you also
route transcription to the cloud, that is the part that adds up, and it is billed per minute
of audio — VAD trimming keeps that number down by never sending silence.

VoxFlow tracks cost per dictation, projects the month, and can warn you at 50, 80, and 100
percent of a cap you set.
