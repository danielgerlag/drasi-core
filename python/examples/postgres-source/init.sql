-- Create sample tables
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    customer_name VARCHAR(100) NOT NULL,
    product VARCHAR(100) NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    total_amount DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create publication for logical replication
CREATE PUBLICATION drasi_pub FOR TABLE orders;

-- Insert some initial data
INSERT INTO orders (customer_name, product, quantity, total_amount, status)
VALUES
    ('Alice', 'Laptop', 1, 1299.99, 'completed'),
    ('Bob', 'Keyboard', 2, 149.98, 'pending'),
    ('Charlie', 'Monitor', 1, 599.99, 'shipped');
