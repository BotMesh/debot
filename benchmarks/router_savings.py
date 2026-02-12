import argparse
import json
from dataclasses import dataclass
from typing import Optional

from datasets import load_dataset, get_dataset_config_names, get_dataset_split_names


@dataclass
class DatasetSpec:
    name: str
    config: Optional[str] = None
    split: Optional[str] = None
    gated: bool = False


DATASETS = [
    DatasetSpec("gaia-benchmark/GAIA", split="dev", gated=True),
    DatasetSpec("UKPLab/DARA-Agentbench", split="train"),
    DatasetSpec("Simia-Agent/Simia-AgentBench-SFT-15k", split="train"),
    DatasetSpec("SWE-bench/SWE-bench", split="test"),
    DatasetSpec("cais/mmlu", split="test"),
    DatasetSpec("oieieio/gsm8k", config="main", split="test"),
]

PREFERRED_FIELDS = [
    "question",
    "prompt",
    "instruction",
    "input",
    "query",
    "problem",
    "task",
    "text",
]


def resolve_split(name: str, config: Optional[str], preferred: Optional[str]) -> str:
    try:
        splits = get_dataset_split_names(name, config) if config else get_dataset_split_names(name)
    except Exception:
        return preferred or "train"
    if preferred and preferred in splits:
        return preferred
    return splits[0] if splits else (preferred or "train")


def pick_text(row: dict) -> tuple[Optional[str], Optional[str]]:
    for key in PREFERRED_FIELDS:
        val = row.get(key)
        if isinstance(val, str) and val.strip():
            return val.strip(), key
    # Fallback: first string field
    for k, val in row.items():
        if isinstance(val, str) and val.strip():
            return val.strip(), k
    return None, None


def load_prompts(spec: DatasetSpec, max_samples: int) -> tuple[list[str], Optional[str]]:
    config = spec.config
    if not config:
        try:
            configs = get_dataset_config_names(spec.name)
            if configs:
                config = configs[0]
        except Exception:
            config = None

    split = resolve_split(spec.name, config, spec.split)
    ds = load_dataset(spec.name, name=config, split=split) if config else load_dataset(spec.name, split=split)

    prompts = []
    picked_field = None
    for row in ds:
        text, field = pick_text(row)
        if text:
            prompts.append(text)
            if picked_field is None and field:
                picked_field = field
        if max_samples > 0 and len(prompts) >= max_samples:
            break
    return prompts, picked_field


def estimate_tokens(text: str, tokenizer=None) -> int:
    if tokenizer:
        try:
            return len(tokenizer.encode(text))
        except Exception:
            pass
    # Naive estimator: 1 token ~= 4 chars
    return max(1, len(text) // 4)


def iter_dataset_configs(spec: DatasetSpec, configs_per_dataset: int) -> list[DatasetSpec]:
    if spec.config:
        return [spec]
    try:
        configs = get_dataset_config_names(spec.name)
    except Exception:
        configs = []
    if not configs:
        return [spec]
    return [DatasetSpec(spec.name, config=c, split=spec.split, gated=spec.gated) for c in configs[:configs_per_dataset]]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-samples", type=int, default=100, help="Samples per dataset config (0=all)")
    parser.add_argument("--configs-per-dataset", type=int, default=3, help="Configs per dataset (if applicable)")
    parser.add_argument(
        "--baseline-models",
        type=str,
        default="anthropic/claude-opus-4-5,openai/o3,openai/gpt-4o-mini",
        help="Comma-separated baseline models",
    )
    args = parser.parse_args()

    try:
        import debot_rust
    except Exception:
        raise SystemExit("debot_rust not available. Build the extension first.")

    baseline_models = [m.strip() for m in args.baseline_models.split(",") if m.strip()]
    if not baseline_models:
        raise SystemExit("No baseline models provided.")

    def get_cost(model: str) -> float:
        try:
            getter = getattr(debot_rust, "get_model_cost", None)
            if getter:
                return float(getter(model))
        except Exception:
            pass
        print(f"[WARN] Baseline cost not found for {model}; using $1.00/M as fallback.")
        return 1.0

    baseline_costs = {m: get_cost(m) for m in baseline_models}

    total_tokens = 0
    total_cost_router = 0.0
    total_prompts = 0
    tokenizer = None
    try:
        import tiktoken

        tokenizer = tiktoken.get_encoding("cl100k_base")
    except Exception:
        tokenizer = None

    for base_spec in DATASETS:
        for spec in iter_dataset_configs(base_spec, args.configs_per_dataset):
            try:
                prompts, field = load_prompts(spec, args.max_samples)
            except Exception as e:
                print(f"[WARN] {spec.name} skipped: {e}")
                continue
            if prompts:
                print(
                    f"[INFO] {spec.name} config={spec.config or '-'} field={field or '-'} "
                    f"sample={prompts[0][:120]!r}"
                )

            for prompt in prompts:
                tokens = estimate_tokens(prompt, tokenizer=tokenizer)
                total_tokens += tokens
                total_prompts += 1

                decision = json.loads(debot_rust.route_text(prompt, 4096))
                router_cost = float(decision.get("cost_estimate", 1.0))
                total_cost_router += tokens / 1_000_000 * router_cost

    if total_prompts == 0:
        print("No prompts available.")
        return

    print("==== Router Token Cost Savings (Estimated) ====")
    print(f"prompts: {total_prompts}")
    print(f"tokens (estimated): {total_tokens}")
    print(f"router cost:   ${total_cost_router:.6f}")
    for model, cost_per_m in baseline_costs.items():
        total_cost_baseline = total_tokens / 1_000_000 * cost_per_m
        savings = total_cost_baseline - total_cost_router
        savings_pct = (savings / total_cost_baseline * 100) if total_cost_baseline > 0 else 0.0
        print(f"baseline model: {model}")
        print(f"baseline cost: ${total_cost_baseline:.6f}")
        print(f"savings:       ${savings:.6f} ({savings_pct:.2f}%)")


if __name__ == "__main__":
    main()
