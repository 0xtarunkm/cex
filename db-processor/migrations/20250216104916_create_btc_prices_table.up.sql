-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE TABLE btc_prices (
    time    TIMESTAMP WITH TIME ZONE NOT NULL,
    price   DOUBLE PRECISION,
    volume  DOUBLE PRECISION,
    currency_code   VARCHAR(10)
);

SELECT create_hypertable('btc_prices', 'time', 'price', 2);

CREATE MATERIALIZED VIEW IF NOT EXISTS klines_1m AS
SELECT
    time_bucket('1 minute', time) AS bucket,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close,
    sum(volume) AS volume,
    currency_code
FROM btc_prices
GROUP BY bucket, currency_code;

CREATE MATERIALIZED VIEW IF NOT EXISTS klines_1h AS
SELECT
    time_bucket('1 hour', time) AS bucket,
    first(price, time) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, time) AS close,
    sum(volume) AS volume,
    currency_code
FROM btc_prices
GROUP BY bucket, currency_code;

CREATE MATERIALIZED VIEW IF NOT EXISTS klines_1w AS
SELECT
    time_bucket('1 week', time) AS bucket,
    first(price, time) AS open,
    max(price) AS high,
    MIN(price) AS low,
    last(price, time) AS close,
    sum(volume) AS volume,
    currency_code
FROM btc_prices
GROUP BY bucket, currency_code;