-- Create payments table for transaction processing
-- Migration: 20250828002_create_payments_table

CREATE TABLE payments (
    id VARCHAR(36) PRIMARY KEY,
    account_id VARCHAR(36) NOT NULL,
    amount_cents BIGINT NOT NULL,
    currency_code CHAR(3) NOT NULL DEFAULT 'JPY',
    payment_method ENUM('CARD', 'BANK_TRANSFER', 'DIGITAL_WALLET') NOT NULL,
    payment_status ENUM('PENDING', 'PROCESSING', 'SUCCESS', 'FAILED', 'CANCELLED', 'REFUNDED') NOT NULL DEFAULT 'PENDING',
    reference_number VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    customer_email VARCHAR(255),
    metadata JSON,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    
    -- Indexes for performance
    INDEX idx_payments_account_id (account_id),
    INDEX idx_payments_status (payment_status),
    INDEX idx_payments_reference (reference_number),
    INDEX idx_payments_created_at (created_at),
    INDEX idx_payments_customer_email (customer_email)
);