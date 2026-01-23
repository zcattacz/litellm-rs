#!/usr/bin/env python3
"""
Model Sync Script for litellm-rs

This script fetches the latest model information from OpenRouter API
and generates a report of models that need to be updated in the codebase.

Usage:
    python scripts/sync_models.py [--update] [--provider PROVIDER]

Options:
    --update        Generate Rust code snippets for new models
    --provider      Filter by provider (openai, anthropic, google, deepseek, etc.)
    --output        Output format: report (default), json, rust
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime
from typing import Any, Dict, List, Optional
from urllib.request import urlopen
from urllib.error import URLError

OPENROUTER_API = "https://openrouter.ai/api/v1/models"

# Provider mapping from OpenRouter to our codebase
PROVIDER_MAPPING = {
    "openai": "openai",
    "anthropic": "anthropic",
    "google": "gemini",
    "deepseek": "deepseek",
    "meta-llama": "meta_llama",
    "mistralai": "mistral",
    "cohere": "cohere",
    "x-ai": "xai",
    "amazon": "amazon_nova",
    "qwen": "qwen",
}

# Models we track (add more as needed)
TRACKED_PROVIDERS = [
    "openai", "anthropic", "google", "deepseek",
    "meta-llama", "mistralai", "cohere", "x-ai"
]


def fetch_openrouter_models() -> List[Dict[str, Any]]:
    """Fetch all models from OpenRouter API."""
    try:
        with urlopen(OPENROUTER_API, timeout=30) as response:
            data = json.loads(response.read().decode())
            return data.get("data", [])
    except URLError as e:
        print(f"Error fetching models: {e}", file=sys.stderr)
        sys.exit(1)


def parse_model_info(model: Dict[str, Any]) -> Dict[str, Any]:
    """Parse model information from OpenRouter format."""
    model_id = model.get("id", "")
    parts = model_id.split("/")
    provider = parts[0] if len(parts) > 1 else "unknown"
    name = parts[1] if len(parts) > 1 else model_id

    pricing = model.get("pricing", {})

    return {
        "id": model_id,
        "provider": provider,
        "name": name,
        "display_name": model.get("name", name),
        "context_length": model.get("context_length", 0),
        "input_cost_per_million": float(pricing.get("prompt", 0)) * 1_000_000,
        "output_cost_per_million": float(pricing.get("completion", 0)) * 1_000_000,
        "supports_tools": "tools" in model.get("supported_parameters", []),
        "supports_vision": "vision" in model.get("architecture", {}).get("modality", ""),
        "created": model.get("created"),
    }


def group_by_provider(models: List[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    """Group models by provider."""
    grouped: Dict[str, List[Dict[str, Any]]] = {}
    for model in models:
        provider = model["provider"]
        if provider not in grouped:
            grouped[provider] = []
        grouped[provider].append(model)
    return grouped


def generate_report(models: List[Dict[str, Any]], provider_filter: Optional[str] = None) -> str:
    """Generate a markdown report of models."""
    grouped = group_by_provider(models)

    lines = [
        "# OpenRouter Model Report",
        f"Generated: {datetime.now().isoformat()}",
        "",
    ]

    for provider in sorted(grouped.keys()):
        if provider_filter and provider != provider_filter:
            continue
        if provider not in TRACKED_PROVIDERS:
            continue

        provider_models = grouped[provider]
        lines.append(f"## {provider.upper()}")
        lines.append("")
        lines.append("| Model | Context | Input $/M | Output $/M | Tools | Vision |")
        lines.append("|-------|---------|-----------|------------|-------|--------|")

        for model in sorted(provider_models, key=lambda x: x["name"]):
            tools = "✓" if model["supports_tools"] else ""
            vision = "✓" if model["supports_vision"] else ""
            lines.append(
                f"| {model['name']} | {model['context_length']:,} | "
                f"${model['input_cost_per_million']:.2f} | "
                f"${model['output_cost_per_million']:.2f} | "
                f"{tools} | {vision} |"
            )
        lines.append("")

    return "\n".join(lines)


def generate_rust_snippet(model: Dict[str, Any]) -> str:
    """Generate Rust code snippet for a model."""
    return f'''
            // {model['display_name']}
            (
                "{model['name']}",
                "{model['display_name']}",
                {model['context_length']},
                Some(16384),
                {model['input_cost_per_million'] / 1_000_000:.6f}, // ${model['input_cost_per_million']:.2f}/1M input
                {model['output_cost_per_million'] / 1_000_000:.6f}, // ${model['output_cost_per_million']:.2f}/1M output
            ),'''


def main():
    parser = argparse.ArgumentParser(description="Sync models from OpenRouter")
    parser.add_argument("--provider", help="Filter by provider")
    parser.add_argument("--output", choices=["report", "json", "rust"], default="report")
    parser.add_argument("--update", action="store_true", help="Generate Rust code snippets")
    args = parser.parse_args()

    print("Fetching models from OpenRouter...", file=sys.stderr)
    raw_models = fetch_openrouter_models()
    models = [parse_model_info(m) for m in raw_models]

    print(f"Found {len(models)} models", file=sys.stderr)

    if args.output == "json":
        if args.provider:
            models = [m for m in models if m["provider"] == args.provider]
        print(json.dumps(models, indent=2))
    elif args.output == "rust":
        if args.provider:
            models = [m for m in models if m["provider"] == args.provider]
        for model in models:
            print(generate_rust_snippet(model))
    else:
        print(generate_report(models, args.provider))


if __name__ == "__main__":
    main()
