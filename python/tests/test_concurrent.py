"""Tests for thread safety: multiple asyncio tasks sharing DrasiLib."""

import asyncio

from .conftest import build_simple_lib, make_person_props


async def test_concurrent_inserts():
    """Multiple asyncio tasks can push data through the same source handle concurrently."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="concurrent-1")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    num_tasks = 10

    async def insert_task(idx: int):
        props = make_person_props(f"User{idx}", age=20 + idx)
        await handle.send_node_insert(f"c{idx}", ["Person"], props)

    tasks = [asyncio.create_task(insert_task(i)) for i in range(num_tasks)]
    await asyncio.gather(*tasks)

    results = []
    try:
        for _ in range(num_tasks):
            r = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
            results.append(r)
    except (asyncio.TimeoutError, StopAsyncIteration):
        pass

    assert len(results) == num_tasks

    await lib.stop()


async def test_concurrent_read_and_write():
    """One task writes data while another reads results concurrently."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="concurrent-2")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    results = []
    insert_count = 5
    done = asyncio.Event()

    async def writer():
        for i in range(insert_count):
            props = make_person_props(f"Writer{i}")
            await handle.send_node_insert(f"w{i}", ["Person"], props)
            await asyncio.sleep(0.05)
        done.set()

    async def reader():
        try:
            while not done.is_set() or len(results) < insert_count:
                r = await asyncio.wait_for(stream.__anext__(), timeout=3.0)
                results.append(r)
        except (asyncio.TimeoutError, StopAsyncIteration):
            pass

    await asyncio.gather(
        asyncio.create_task(writer()),
        asyncio.create_task(reader()),
    )

    assert len(results) == insert_count

    await lib.stop()


async def test_concurrent_status_checks():
    """Multiple tasks querying status concurrently should not race."""
    lib, _, _ = await build_simple_lib(lib_id="concurrent-3")
    await lib.start()
    await asyncio.sleep(0.1)

    async def check_status():
        return await lib.is_running()

    tasks = [asyncio.create_task(check_status()) for _ in range(20)]
    results = await asyncio.gather(*tasks)

    assert all(r is True for r in results)

    await lib.stop()
