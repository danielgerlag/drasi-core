"""Simulate database changes to trigger Drasi query updates.
Run this in a separate terminal while main.py is running."""
import asyncio
import asyncpg


async def main():
    conn = await asyncpg.connect(
        host="localhost", port=5432,
        database="drasi_example", user="drasi", password="drasi_pass"
    )
    print("Connected to PostgreSQL. Simulating changes...")

    # Insert a new high-value order (triggers ADD)
    await conn.execute(
        "INSERT INTO orders (customer_name, product, quantity, total_amount, status) "
        "VALUES ($1, $2, $3, $4, $5)",
        "Diana", "Tablet", 1, 799.99, "pending"
    )
    print("  Inserted order for Diana (total_amount=799.99 > 500 → ADD)")
    await asyncio.sleep(2)

    # Update Diana's order (triggers UPDATE)
    await conn.execute(
        "UPDATE orders SET status = 'shipped', total_amount = 999.99 WHERE customer_name = 'Diana'"
    )
    print("  Updated Diana's order (total_amount=999.99, status=shipped → UPDATE)")
    await asyncio.sleep(2)

    # Delete Diana's order (triggers DELETE)
    await conn.execute("DELETE FROM orders WHERE customer_name = 'Diana'")
    print("  Deleted Diana's order (→ DELETE)")

    await conn.close()
    print("Done simulating changes")


if __name__ == "__main__":
    asyncio.run(main())
