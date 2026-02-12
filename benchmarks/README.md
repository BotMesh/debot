# Benchmarks

This directory provides scripts to **download and sanity-check open datasets** for evaluating agent capability.  
Data is **downloaded on demand** via Hugging Face `datasets` and is **not** committed to the repo.

## Included datasets

1. **GAIA** (tool-augmented agent benchmark)  
   Dataset: `gaia-benchmark/GAIA` (gated). You must accept the dataset terms on Hugging Face.

2. **AgentBench-format dataset (DARA)**  
   Dataset: `UKPLab/DARA-Agentbench`

3. **AgentBench synthetic tool-use dataset**  
   Dataset: `Simia-Agent/Simia-AgentBench-SFT-15k`

4. **SWE-bench** (software engineering issues)  
   Dataset: `SWE-bench/SWE-bench`

5. **MMLU** (general knowledge, multiple choice)  
   Dataset: `cais/mmlu`

6. **GSM8K** (grade-school math)  
   Dataset: `oieieio/gsm8k`

## Usage

Install deps and run all downloads:

```bash
make benchmark
```

Estimate token cost savings from the router (baseline: strongest model):

```bash
make benchmark-router
```

You can override baselines and increase coverage:

```bash
python benchmarks/router_savings.py --max-samples 200 --configs-per-dataset 5 \
  --baseline-models anthropic/claude-opus-4-5,openai/o3,openai/gpt-4o-mini
```

## Interpreting results

- If you compare against a **very cheap baseline** (e.g. `openai/gpt-4o-mini`), it's normal for router cost to be **higher**.
  The router intentionally routes harder prompts to more capable (and more expensive) models.
- For a meaningful "savings" number, use a **strong baseline** like `anthropic/claude-opus-4-5` or `openai/o3`.
This runs a **lightweight sanity-check**:
- downloads each dataset
- prints basic schema + sample counts
- optionally limits samples

## Gated datasets (GAIA)

GAIA is gated on Hugging Face.  
If you see an auth error, do:

1. Accept the dataset terms on HF
2. Log in:
   ```bash
   huggingface-cli login
   ```

## Notes

- These scripts do **not** run full agent evaluations (no cost-heavy LLM loops).  
- They are meant to verify dataset access and structure before deeper benchmarking.
