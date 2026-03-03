# Performance Tuning Guide

Quick reference for squeezing tokens/sec out of your models.

## GPU Layers (`gpu-layers`)

**The Rule**: Push it until it fails, then back off a bit.

```bash
spellbook serve model-name --gpu-layers 999  # Start insanely high
# If it OOMs, drop by 5-10 and try again
```

Your GPU VRAM is the limit. A 24GB GPU might fit:
- Small models (3-7B): 99+ layers (entire model on GPU)
- Medium models (30B): 40-60 layers
- Large models (70B+): 20-40 layers

Once you find the max that fits, you've probably found your sweet spot. Going lower usually just makes things slower.

## Context Size (`context`)

**Default is fine for most use cases.** Bigger context = more VRAM used.

- Start with **32k** for general use
- If you hit OOM errors, drop to 16k
- If you want long conversations, push toward 65k (only if VRAM allows)
- **Don't optimize this without a reason** — the default works

## Batch Size (`--batch-size` / `-b`)

**Default (2048) is fine. Skip this unless you notice slowness.**

If throughput is low:
- Try doubling batch size (4096 or 8192)
- Or drop micro-batch (`-ub`) to half (256)

Rules of thumb:
- Bigger batch = faster for multiple requests
- Smaller batch = lower latency for single requests
- Experiment only if you're serving multiple users at once

## Continuous Batching (`--cont-batching`)

**Try it.** It helps with multiple concurrent requests.

```bash
spellbook serve model-name --cont-batching
```

Enables dynamic request batching. Good for serving, minimal downside.

## Cache Prompt (`--cache-prompt`)

**Not enabled by default.** Try it if you're reusing prompts.

Useful for:
- Chat applications (same system prompt)
- Batch processing similar queries

Otherwise skip it.

## Flash Attention (`-fa` / `--flash-attention`)

**For iGPU: Disable it** (`--no-flash-attn`).

Flash Attention on integrated GPU is 12-19% slower and causes more crashes due to memory bandwidth limitations. Not worth it.

For discrete GPU: Worth trying if model supports it.

## iGPU with 32GB DDR5 + Dynamic UMA

**Setup notes:**
- Let UMA dynamically allocate (don't fix to 6GB)
- iGPU can pull from full 32GB pool as needed
- Disable `--flash-attention` (makes it slower, not faster on iGPU)
- Always use `--mmap`

**GPU Layer strategy:**
- Start with 45-50 layers, push higher until you hit slowdown
- Qwen 3 Coder 30B (MoE, ~3.3B active) will fit comfortably
- You're memory-bandwidth limited, not capacity limited

**Qwen 3 Coder 30B baseline:**
- Q4_K_M quantization: ~18.6GB
- GPU layers: 45-60 (test your sweet spot)
- Context: 65536 (you have the room)
- Expected: 12-18 tokens/sec (prompt processing will be 2-3x faster)

**Known issue:** llama.cpp may misread UMA allocation—if you see warnings about shared memory, ignore them if you're using dynamic allocation.

## CPU Threads (`--threads` / `-t`)

**Default is usually optimal.** Only change if you know what you're doing.

- Default: auto-detected (typically cores - 2)
- If you're seeing high CPU usage, try lowering by 2-4
- If CPU is idle and generation is slow, try raising

## Testing Setup

Simple workflow:

1. **Start**: Use config defaults (gpu-layers, context, threads)
2. **Push GPU layers**: Increment until OOM, back off 5 layers
3. **Measure baseline**: Generate 1000 tokens, note tokens/sec
4. **Try flash-attention**: Measure again, keep if faster
5. **Try cont-batching**: Only if serving multiple users
6. **Stop**: You probably have diminishing returns

## Monitoring

Use `--perf` to see internal timings:

```bash
spellbook serve model-name --perf
```

Look for which step is slowest (load, prefill, generation) to know where to focus.

## Quick Checklist

- [ ] Push gpu-layers to the max your VRAM allows
- [ ] Enable flash-attention if model supports it
- [ ] Enable cont-batching if serving multiple users
- [ ] Leave context, batch size, threads alone unless something breaks
- [ ] Measure before/after changes (same prompt, same token count)
