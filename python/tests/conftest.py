"""Shared fixtures and helpers for Drasi Python integration tests."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

import pytest
from drasi_lib import DrasiLibBuilder, Query
from drasi_reaction_application import ApplicationReaction, ApplicationReactionHandle
from drasi_source_application import (
    ApplicationSource,
    ApplicationSourceHandle,
    PropertyMapBuilder,
)

if TYPE_CHECKING:
    from drasi_lib import DrasiLib
    from drasi_source_application import PropertyMap


@pytest.fixture
def event_loop():
    """Provide a fresh event loop for each test."""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


async def build_simple_lib(
    lib_id: str = "test",
    source_id: str = "test-source",
    query_id: str = "test-query",
    reaction_id: str = "test-reaction",
    cypher: str = "MATCH (n:Person) RETURN n.name",
    auto_start_query: bool = True,
) -> tuple[
    "DrasiLib",
    ApplicationSourceHandle,
    ApplicationReactionHandle,
]:
    """Build a DrasiLib with a single ApplicationSource, Cypher query, and ApplicationReaction.

    Returns (lib, source_handle, reaction_handle).
    """
    source = ApplicationSource(source_id)
    source_handle = source.get_handle()

    reaction_builder = ApplicationReaction.builder(reaction_id)
    reaction_builder.with_query(query_id)
    reaction, reaction_handle = reaction_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id(lib_id)
    lib_builder.with_source(source.into_source_wrapper())

    q = Query.cypher(query_id)
    q.query(cypher)
    q.from_source(source_id)
    q.auto_start(auto_start_query)
    lib_builder.with_query(q.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    return lib, source_handle, reaction_handle


def make_person_props(name: str, age: int | None = None) -> "PropertyMap":
    """Build a PropertyMap for a Person node."""
    builder = PropertyMapBuilder()
    builder.with_string("name", name)
    if age is not None:
        builder.with_integer("age", age)
    return builder.build()
