import argparse
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
    DatasetSpec("cais/mmlu", config="abstract_algebra", split="test"),
    DatasetSpec("oieieio/gsm8k", config="main", split="test"),
]


def resolve_split(name: str, config: Optional[str], preferred: Optional[str]) -> str:
    try:
        splits = get_dataset_split_names(name, config) if config else get_dataset_split_names(name)
    except Exception:
        return preferred or "train"
    if preferred and preferred in splits:
        return preferred
    return splits[0] if splits else (preferred or "train")


def load_one(spec: DatasetSpec, max_samples: int) -> None:
    print(f"\n==> {spec.name}")
    if spec.gated:
        print("   (gated dataset; requires HF access)")

    config = spec.config
    if not config:
        try:
            configs = get_dataset_config_names(spec.name)
            if configs:
                config = configs[0]
        except Exception:
            config = None

    split = resolve_split(spec.name, config, spec.split)
    print(f"   config={config or '-'} split={split}")

    ds = load_dataset(spec.name, name=config, split=split) if config else load_dataset(spec.name, split=split)
    print(f"   rows={len(ds)} columns={ds.column_names}")
    if max_samples > 0:
        ds = ds.select(range(min(max_samples, len(ds))))
        print(f"   sample_rows={len(ds)}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-samples", type=int, default=5, help="Limit samples per dataset (0=all)")
    args = parser.parse_args()

    for spec in DATASETS:
        try:
            load_one(spec, args.max_samples)
        except Exception as e:
            print(f"   ERROR: {e}")


if __name__ == "__main__":
    main()
