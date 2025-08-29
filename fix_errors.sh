#!/bin/bash

# Fix ApplicationError usage in payment use cases
files=(
    "src/application/usecases/process_payment_usecase.rs"
    "src/application/usecases/refund_payment_usecase.rs"
)

for file in "${files[@]}"; do
    echo "Fixing $file..."
    
    # Fix BusinessRuleViolation -> Domain conversion
    sed -i 's/ApplicationError::BusinessRuleViolation {[^}]*message: e\.to_string(),[^}]*}/ApplicationError::Domain(e)/g' "$file"
    
    # Fix RepositoryError field syntax
    sed -i 's/ApplicationError::RepositoryError {[^}]*message: e\.to_string(),[^}]*}/ApplicationError::RepositoryError(e.to_string())/g' "$file"
    
    # Fix NotFound entity -> resource
    sed -i 's/entity: "Payment"/resource: "Payment"/g' "$file"
    
    # Fix ValidationError field syntax  
    sed -i 's/ApplicationError::ValidationError {[^}]*field: "\([^"]*\)"[^}]*message: \([^}]*\)[^}]*}/ApplicationError::ValidationError(\2)/g' "$file"
done

echo "Done!"