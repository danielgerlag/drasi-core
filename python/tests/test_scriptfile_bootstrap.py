"""Tests for ScriptFileBootstrapProvider: building and configuring bootstrap providers."""

import json
import os
import tempfile

from drasi_bootstrap_scriptfile import PyScriptFileBootstrapProvider


def _write_bootstrap_jsonl(path: str, nodes: list[dict]) -> None:
    """Write a JSONL bootstrap file with a header, node records, and finish marker."""
    with open(path, "w") as f:
        header = {"type": "header", "source_id": "test-source", "version": "1.0"}
        f.write(json.dumps(header) + "\n")

        for node in nodes:
            record = {
                "type": "node",
                "id": node["id"],
                "labels": node["labels"],
                "properties": node["properties"],
            }
            f.write(json.dumps(record) + "\n")

        finish = {"type": "finish"}
        f.write(json.dumps(finish) + "\n")


async def test_bootstrap_builder_single_file():
    """Build a ScriptFileBootstrapProvider with a single file via builder."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bootstrap_path = os.path.join(tmpdir, "bootstrap.jsonl")
        _write_bootstrap_jsonl(
            bootstrap_path,
            [
                {"id": "p1", "labels": ["Person"], "properties": {"name": "Alice"}},
                {"id": "p2", "labels": ["Person"], "properties": {"name": "Bob"}},
            ],
        )

        builder = PyScriptFileBootstrapProvider.builder()
        builder.with_file(bootstrap_path)
        bootstrap = builder.build()
        assert bootstrap is not None

        wrapper = bootstrap.into_bootstrap_wrapper()
        assert wrapper is not None


async def test_bootstrap_with_paths_static_method():
    """Use the with_paths static constructor."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bootstrap_path = os.path.join(tmpdir, "data.jsonl")
        _write_bootstrap_jsonl(
            bootstrap_path,
            [{"id": "x1", "labels": ["Person"], "properties": {"name": "Xander"}}],
        )

        bootstrap = PyScriptFileBootstrapProvider.with_paths([bootstrap_path])
        assert bootstrap is not None

        wrapper = bootstrap.into_bootstrap_wrapper()
        assert wrapper is not None


async def test_bootstrap_multiple_files():
    """ScriptFileBootstrapProvider handles multiple file paths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        path1 = os.path.join(tmpdir, "part1.jsonl")
        path2 = os.path.join(tmpdir, "part2.jsonl")

        _write_bootstrap_jsonl(
            path1,
            [{"id": "p1", "labels": ["Person"], "properties": {"name": "Alice"}}],
        )
        _write_bootstrap_jsonl(
            path2,
            [{"id": "p2", "labels": ["Person"], "properties": {"name": "Bob"}}],
        )

        builder = PyScriptFileBootstrapProvider.builder()
        builder.with_file_paths([path1, path2])
        bootstrap = builder.build()
        assert bootstrap is not None
