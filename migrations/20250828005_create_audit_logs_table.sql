-- Create audit_logs table for comprehensive audit trails
-- Migration: 20250828005_create_audit_logs_table

CREATE TABLE audit_logs (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    entity_type ENUM('ACCOUNT', 'PAYMENT', 'USER', 'SYSTEM') NOT NULL,
    entity_id VARCHAR(36) NOT NULL,
    operation ENUM('CREATE', 'READ', 'UPDATE', 'DELETE') NOT NULL,
    old_values JSON,
    new_values JSON,
    user_id VARCHAR(36),
    ip_address VARCHAR(45),
    user_agent TEXT,
    request_id VARCHAR(36),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for performance
    INDEX idx_audit_logs_entity (entity_type, entity_id),
    INDEX idx_audit_logs_operation (operation),
    INDEX idx_audit_logs_user_id (user_id),
    INDEX idx_audit_logs_created_at (created_at),
    INDEX idx_audit_logs_request_id (request_id)
);