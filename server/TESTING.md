# Notes Server Testing Guide

This project has two separate test suites:

## Unit Tests

Located in: `src/handlers/notes.rs`
Tests helper functions without database dependencies.

**Run unit tests:**
```bash
cargo test --lib
```

These tests include:
- `test_success_response` - Tests API success response formatting
- `test_error_response` - Tests API error response formatting  
- `test_error_tuple` - Tests error tuple helper function

## Integration Tests

Located in: `tests/handlers_integration_tests.rs`
Tests actual database operations and requires PostgreSQL.

### Automated Setup

Use the Python setup script to prepare the test environment:

```bash
# Standard setup
python3 scripts/setup-integration-tests.py

# Verify existing setup
python3 scripts/setup-integration-tests.py --verify-only

# Clean setup (drops and recreates database)
python3 scripts/setup-integration-tests.py --clean

# Custom configuration
DB_PASSWORD=mypass TEST_DB=custom_test python3 scripts/setup-integration-tests.py
```

**Prerequisites:**
1. Python 3.6+ with psycopg2: `pip install psycopg2-binary`
2. PostgreSQL running on localhost:5432
3. Database access credentials (postgres/password by default)

### Manual Setup

If you prefer manual setup:
1. PostgreSQL running on localhost:5432
2. Create test database: `CREATE DATABASE notes_test;`
3. Default credentials: postgres/password (or set `TEST_DATABASE_URL`)

**Run integration tests:**
```bash
# Run only integration tests
cargo test --test handlers_integration_tests

# Run with custom database URL
TEST_DATABASE_URL="postgres://user:pass@localhost:5432/test_db" cargo test --test handlers_integration_tests
```

**Integration tests include:**
- CRUD operations for notes (create, read, update, delete)
- Validation error handling
- Database error scenarios
- Response formatting verification

## Run All Tests

```bash
# Run unit tests only (no database required)
cargo test --lib

# Run integration tests only (requires database)
cargo test --test handlers_integration_tests

# Run both (requires database for integration tests)
cargo test --lib && cargo test --test handlers_integration_tests
```

## Environment Variables

Configure the test environment with these variables:

```bash
export DB_HOST="localhost"           # Database host
export DB_PORT="5432"               # Database port  
export DB_USER="postgres"           # Database user
export DB_PASSWORD="password"       # Database password
export TEST_DB="notes_test"         # Test database name
export TEST_DATABASE_URL="postgres://user:pass@host:port/db"  # Full connection URL
```

## Troubleshooting

### PostgreSQL Not Running
```bash
# macOS (Homebrew)
brew services start postgresql

# Ubuntu/Debian
sudo systemctl start postgresql

# Check if running
python3 scripts/setup-integration-tests.py --verify-only
```

### Permission Issues
```bash
# Grant privileges to test database
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE notes_test TO postgres;"
```

### Clean Reset
```bash
# Drop and recreate everything
python3 scripts/setup-integration-tests.py --clean
```