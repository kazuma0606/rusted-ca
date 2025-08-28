-- Create payment_events table for event sourcing and audit trails
-- Migration: 20250828003_create_payment_events_table

CREATE TABLE payment_events (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    payment_id VARCHAR(36) NOT NULL,
    event_type ENUM('CREATED', 'AUTHORIZED', 'CAPTURED', 'FAILED', 'CANCELLED', 'REFUNDED', 'DISPUTED') NOT NULL,
    event_data JSON NOT NULL,
    user_id VARCHAR(36),
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE CASCADE,
    
    -- Indexes for performance
    INDEX idx_payment_events_payment_id (payment_id),
    INDEX idx_payment_events_event_type (event_type),
    INDEX idx_payment_events_created_at (created_at),
    INDEX idx_payment_events_user_id (user_id)
);