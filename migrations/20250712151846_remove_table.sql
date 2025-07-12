-- Add migration script here
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    email VARCHAR(255),
    name VARCHAR(100),
    password_hash VARCHAR(255),
    phone VARCHAR(20),
    birth_date DATE,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);