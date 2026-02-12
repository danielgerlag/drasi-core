"""Tests for PropertyMapBuilder: building property maps with various types."""

import pytest
from drasi_source_application import PyPropertyMapBuilder


async def test_build_with_string():
    """Build a PropertyMap with string values."""
    builder = PyPropertyMapBuilder()
    builder.with_string("name", "Alice")
    props = builder.build()
    assert props is not None


async def test_build_with_integer():
    """Build a PropertyMap with integer values."""
    builder = PyPropertyMapBuilder()
    builder.with_integer("age", 42)
    props = builder.build()
    assert props is not None


async def test_build_with_float():
    """Build a PropertyMap with float values."""
    builder = PyPropertyMapBuilder()
    builder.with_float("score", 99.5)
    props = builder.build()
    assert props is not None


async def test_build_with_bool():
    """Build a PropertyMap with boolean values."""
    builder = PyPropertyMapBuilder()
    builder.with_bool("active", True)
    props = builder.build()
    assert props is not None


async def test_build_with_null():
    """Build a PropertyMap with null values."""
    builder = PyPropertyMapBuilder()
    builder.with_null("optional_field")
    props = builder.build()
    assert props is not None


async def test_build_with_all_types():
    """Build a PropertyMap with all supported types in one map."""
    builder = PyPropertyMapBuilder()
    builder.with_string("name", "Alice")
    builder.with_integer("age", 30)
    builder.with_float("score", 95.5)
    builder.with_bool("active", True)
    builder.with_null("nickname")
    props = builder.build()
    assert props is not None


async def test_build_empty_properties():
    """Build an empty PropertyMap."""
    builder = PyPropertyMapBuilder()
    props = builder.build()
    assert props is not None


async def test_builder_consumed_after_build():
    """After calling build(), the builder's internal state is consumed.

    A second build() call should raise an error since the inner HashMap
    has been taken.
    """
    builder = PyPropertyMapBuilder()
    builder.with_string("key", "value")
    props = builder.build()
    assert props is not None

    with pytest.raises(Exception):
        builder.build()
