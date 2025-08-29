-- Create accounts table for merchant account management
-- Migration: 20250828001_create_accounts_table

CREATE TABLE accounts (
    id VARCHAR(36) PRIMARY KEY,
    merchant_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    account_status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    balance_cents BIGINT NOT NULL DEFAULT 0,
    currency_code CHAR(3) NOT NULL DEFAULT 'JPY',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- Indexes for performance
    INDEX idx_accounts_email (email),
    INDEX idx_accounts_status (account_status),
    INDEX idx_accounts_created_at (created_at)
);