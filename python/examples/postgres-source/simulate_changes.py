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

    # Insert new orders
    await conn.execute(
        "INSERT INTO orders (customer_name, product, quantity, total_amount, status) "
        "VALUES ($1, $2, $3, $4, $5)",
        "Diana", "Tablet", 1, 799.99, "pending"
    )
    print("  Inserted order for Diana")
    await asyncio.sleep(1)

    # Update existing order
    await conn.execute(
        "UPDATE orders SET status = 'shipped', total_amount = 1399.99 WHERE customer_name = 'Alice'"
    )
    print("  Updated Alice's order")
    await asyncio.sleep(1)

    # Delete an order
    await conn.execute("DELETE FROM orders WHERE customer_name = 'Bob'")
    print("  Deleted Bob's order")

    await conn.close()
    print("Done simulating changes")


if __name__ == "__main__":
    asyncio.run(main())
