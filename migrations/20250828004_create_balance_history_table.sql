-- Create balance_history table for tracking all balance changes
-- Migration: 20250828004_create_balance_history_table

CREATE TABLE balance_history (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    account_id VARCHAR(36) NOT NULL,
    payment_id VARCHAR(36),
    change_amount_cents BIGINT NOT NULL,
    balance_before_cents BIGINT NOT NULL,
    balance_after_cents BIGINT NOT NULL,
    operation_type ENUM('CHARGE', 'REFUND', 'SETTLEMENT', 'FEE', 'ADJUSTMENT') NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE SET NULL,
    
    -- Indexes for performance
    INDEX idx_balance_history_account_id (account_id),
    INDEX idx_balance_history_payment_id (payment_id),
    INDEX idx_balance_history_operation_type (operation_type),
    INDEX idx_balance_history_created_at (created_at)
);