"""Tests for error handling and error propagation."""

import pytest
from drasi_core import DrasiError
from drasi_lib import DrasiLibBuilder, Query
from drasi_reaction_application import ApplicationReaction
from drasi_source_application import ApplicationSource

from .conftest import build_simple_lib


async def test_invalid_cypher_syntax():
    """An invalid Cypher query should raise an error during build or start."""
    source = ApplicationSource("err-source")

    reaction_builder = ApplicationReaction.builder("err-reaction")
    reaction_builder.with_query("err-query")
    reaction, _ = reaction_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("err-cypher")
    lib_builder.with_source(source.into_source_wrapper())

    q = Query.cypher("err-query")
    q.query("THIS IS NOT VALID CYPHER !!!")
    q.from_source("err-source")
    q.auto_start(True)

    with pytest.raises(Exception):
        lib_builder.with_query(q.build())
        lib = await lib_builder.build()
        await lib.start()


async def test_get_status_nonexistent_source():
    """Querying status of a non-existent source should raise an error."""
    lib, _, _ = await build_simple_lib(lib_id="err-nosrc")
    await lib.start()

    with pytest.raises(Exception):
        await lib.get_source_status("nonexistent-source")

    await lib.stop()


async def test_get_status_nonexistent_query():
    """Querying status of a non-existent query should raise an error."""
    lib, _, _ = await build_simple_lib(lib_id="err-noqry")
    await lib.start()

    with pytest.raises(Exception):
        await lib.get_query_status("nonexistent-query")

    await lib.stop()


async def test_get_status_nonexistent_reaction():
    """Querying status of a non-existent reaction should raise an error."""
    lib, _, _ = await build_simple_lib(lib_id="err-norxn")
    await lib.start()

    with pytest.raises(Exception):
        await lib.get_reaction_status("nonexistent-reaction")

    await lib.stop()


async def test_remove_nonexistent_source():
    """Removing a non-existent source should raise an error."""
    lib, _, _ = await build_simple_lib(lib_id="err-remsrc")
    await lib.start()

    with pytest.raises(Exception):
        await lib.remove_source("nonexistent")

    await lib.stop()


async def test_remove_nonexistent_query():
    """Removing a non-existent query should raise an error."""
    lib, _, _ = await build_simple_lib(lib_id="err-remqry")
    await lib.start()

    with pytest.raises(Exception):
        await lib.remove_query("nonexistent")

    await lib.stop()


async def test_remove_nonexistent_reaction():
    """Removing a non-existent reaction should raise an error."""
    lib, _, _ = await build_simple_lib(lib_id="err-remrxn")
    await lib.start()

    with pytest.raises(Exception):
        await lib.remove_reaction("nonexistent")

    await lib.stop()


async def test_drasi_error_is_exception():
    """DrasiError is a Python Exception subclass."""
    assert issubclass(DrasiError, Exception)
