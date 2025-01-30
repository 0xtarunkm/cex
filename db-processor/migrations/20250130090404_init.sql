DROP TABLE IF EXISTS "sol_prices";

CREATE TABLE "sol_prices" (
    time TIMESTAMP
    WITH
        TIME ZONE NOT NULL,
        price DOUBLE PRECISION,
        volume DOUBLE PRECISION,
        currency_code VARCHAR(10)
);

SELECT
    create_hypertable ('sol_prices', 'time', 'price', 2);
