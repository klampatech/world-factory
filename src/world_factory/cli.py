"""Command-line boundary for generation and persisted-world validation."""

import argparse
import json
import sys
from collections.abc import Sequence
from pathlib import Path
from typing import TextIO

from pydantic import ValidationError

from world_factory.constants import CANONICAL_DEMO_SEED
from world_factory.demo import run_v1_demo
from world_factory.generator import generate_world
from world_factory.models import ClimateClass, WorldConfig, WorldScale
from world_factory.persistence import load_world, save_world
from world_factory.validation import ValidationReport, validate_world

_EXIT_SUCCESS = 0
_EXIT_INVALID_WORLD = 1
_EXIT_OPERATIONAL_ERROR = 2


def main(argv: Sequence[str] | None = None) -> int:
    """Run the World Factory CLI and return a process exit code."""
    parser = _create_parser()
    arguments = parser.parse_args(argv)
    try:
        if arguments.command == "generate":
            return _run_generate(arguments)
        if arguments.command == "validate":
            return _run_validate(arguments)
        if arguments.command == "demo":
            return _run_demo(arguments)
        parser.error("a command is required")
    except (OSError, ValueError, ValidationError) as error:
        _write_json(sys.stderr, {"error": {"code": "operation-failed", "message": str(error)}})
        return _EXIT_OPERATIONAL_ERROR
    return _EXIT_OPERATIONAL_ERROR


def _create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="world-factory")
    commands = parser.add_subparsers(dest="command", required=True)
    generate = commands.add_parser("generate", help="generate and persist a deterministic world")
    generate.add_argument("--seed", type=int, default=CANONICAL_DEMO_SEED)
    generate.add_argument(
        "--scale", choices=[value.value for value in WorldScale], default=WorldScale.SMALL.value
    )
    generate.add_argument(
        "--climate",
        choices=[value.value for value in ClimateClass],
        default=ClimateClass.TEMPERATE.value,
    )
    generate.add_argument("--sentience", action=argparse.BooleanOptionalAction, default=True)
    generate.add_argument("--magic", action=argparse.BooleanOptionalAction, default=False)
    generate.add_argument("--plate-count", type=int, default=12)
    generate.add_argument("--out", type=Path, required=True)
    validate = commands.add_parser("validate", help="strictly load and validate a persisted world")
    validate.add_argument("world", type=Path)
    demo = commands.add_parser(
        "demo", help="run the v1 demo walkthrough (end-to-end world exploration)"
    )
    demo.add_argument("--seed", type=int, default=CANONICAL_DEMO_SEED)
    demo.add_argument(
        "--scale",
        choices=[value.value for value in WorldScale],
        default=WorldScale.LARGE.value,
    )
    demo.add_argument("--out", type=Path, required=True)
    return parser


def _run_generate(arguments: argparse.Namespace) -> int:
    config = WorldConfig(
        seed=arguments.seed,
        scale=WorldScale(arguments.scale),
        climate_class=ClimateClass(arguments.climate),
        sentience_enabled=arguments.sentience,
        magic_enabled=arguments.magic,
        plate_count=arguments.plate_count,
    )
    world = generate_world(config)
    report = validate_world(world)
    if not report.is_valid:
        _write_report(report)
        return _EXIT_INVALID_WORLD
    save_world(world, arguments.out)
    _write_json(
        sys.stdout,
        {"data": {"world_id": world.metadata.world_id, "path": str(arguments.out)}, "error": None},
    )
    return _EXIT_SUCCESS


def _run_validate(arguments: argparse.Namespace) -> int:
    report = validate_world(load_world(arguments.world))
    _write_report(report)
    return _EXIT_SUCCESS if report.is_valid else _EXIT_INVALID_WORLD


def _write_report(report: ValidationReport) -> None:
    _write_json(sys.stdout, {"data": report.model_dump(mode="json"), "error": None})


def _run_demo(arguments: argparse.Namespace) -> int:
    report = run_v1_demo(
        seed=arguments.seed,
        scale=WorldScale(arguments.scale),
    )
    payload: dict[str, object] = {
        "data": report.to_dict(),
        "error": None,
    }
    _write_json(sys.stdout, payload)
    arguments.out.parent.mkdir(parents=True, exist_ok=True)
    arguments.out.write_text(
        json.dumps(payload, allow_nan=False, sort_keys=True, indent=2) + "\n"
    )
    return _EXIT_SUCCESS if report.is_valid else _EXIT_INVALID_WORLD


def _write_json(stream: TextIO, payload: object) -> None:
    print(json.dumps(payload, allow_nan=False, sort_keys=True), file=stream)


if __name__ == "__main__":
    raise SystemExit(main())
