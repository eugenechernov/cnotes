#!/usr/bin/env python3
"""
Integration Test Harness Setup Script
Prepares the PostgreSQL database for running integration tests
"""

import argparse
import os
import subprocess
import sys
import time
from typing import Optional

try:
    import psycopg2
    from psycopg2 import sql
except ImportError:
    print("❌ PostgreSQL adapter 'psycopg2' not found.")
    print("Install it with: pip install psycopg2-binary")
    sys.exit(1)


class Colors:
    """ANSI color codes for terminal output"""
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color


class TestHarnessSetup:
    """Handles setup of PostgreSQL test environment for integration tests"""
    
    def __init__(self):
        # Default configuration
        self.db_host = os.environ.get('DB_HOST', 'localhost')
        self.db_port = int(os.environ.get('DB_PORT', '5432'))
        self.db_user = os.environ.get('DB_USER', 'postgres')
        self.db_password = os.environ.get('DB_PASSWORD', 'password')
        self.test_db = os.environ.get('TEST_DB', 'test_notes_db')
        
    def print_status(self, message: str) -> None:
        """Print blue info message"""
        print(f"{Colors.BLUE}[INFO]{Colors.NC} {message}")
        
    def print_success(self, message: str) -> None:
        """Print green success message"""
        print(f"{Colors.GREEN}[SUCCESS]{Colors.NC} {message}")
        
    def print_warning(self, message: str) -> None:
        """Print yellow warning message"""
        print(f"{Colors.YELLOW}[WARNING]{Colors.NC} {message}")
        
    def print_error(self, message: str) -> None:
        """Print red error message"""
        print(f"{Colors.RED}[ERROR]{Colors.NC} {message}")
        
    def get_connection_params(self, database: str = 'postgres') -> dict:
        """Get connection parameters for psycopg2"""
        return {
            'host': self.db_host,
            'port': self.db_port,
            'user': self.db_user,
            'password': self.db_password,
            'database': database,
            'connect_timeout': 10
        }
        
    def check_postgres(self) -> bool:
        """Check if PostgreSQL is running and accessible"""
        self.print_status("Checking if PostgreSQL is running...")
        
        try:
            conn = psycopg2.connect(**self.get_connection_params())
            conn.close()
            self.print_success(f"PostgreSQL is running on {self.db_host}:{self.db_port}")
            return True
        except psycopg2.Error as e:
            self.print_error(f"Cannot connect to PostgreSQL on {self.db_host}:{self.db_port}")
            self.print_error(f"Error: {e}")
            return False
            
    def start_postgres(self) -> bool:
        """Attempt to start PostgreSQL using various methods"""
        self.print_status("Attempting to start PostgreSQL...")
        
        # Try Homebrew (macOS)
        if self._command_exists('brew'):
            self.print_status("Starting PostgreSQL using Homebrew...")
            if self._run_command(['brew', 'services', 'start', 'postgresql']):
                self.print_success("PostgreSQL started via Homebrew")
                return True
                
        # Try systemctl (systemd systems)
        if self._command_exists('systemctl'):
            self.print_status("Starting PostgreSQL using systemctl...")
            if self._run_command(['sudo', 'systemctl', 'start', 'postgresql']):
                self.print_success("PostgreSQL started via systemctl")
                return True
                
        # Try service command (SysV init systems)
        if self._command_exists('service'):
            self.print_status("Starting PostgreSQL using service...")
            if self._run_command(['sudo', 'service', 'postgresql', 'start']):
                self.print_success("PostgreSQL started via service")
                return True
                
        self.print_error("Could not start PostgreSQL automatically")
        self.print_status("Please start PostgreSQL manually and run this script again")
        return False
        
    def _command_exists(self, command: str) -> bool:
        """Check if a command exists in PATH"""
        try:
            subprocess.run(['which', command], capture_output=True, check=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False
            
    def _run_command(self, command: list, capture_output: bool = True) -> bool:
        """Run a shell command and return success status"""
        try:
            result = subprocess.run(command, capture_output=capture_output, text=True)
            return result.returncode == 0
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False
            
    def database_exists(self, database_name: str) -> bool:
        """Check if a database exists"""
        try:
            conn = psycopg2.connect(**self.get_connection_params())
            cursor = conn.cursor()
            
            cursor.execute(
                "SELECT 1 FROM pg_database WHERE datname = %s",
                (database_name,)
            )
            exists = cursor.fetchone() is not None
            
            cursor.close()
            conn.close()
            return exists
        except psycopg2.Error:
            return False
            
    def create_test_database(self) -> bool:
        """Create the test database if it doesn't exist"""
        self.print_status(f"Creating test database '{self.test_db}'...")
        
        if self.database_exists(self.test_db):
            self.print_warning(f"Database '{self.test_db}' already exists")
            return True
            
        try:
            conn = psycopg2.connect(**self.get_connection_params())
            conn.set_isolation_level(psycopg2.extensions.ISOLATION_LEVEL_AUTOCOMMIT)
            cursor = conn.cursor()
            
            # Create database - using sql.Identifier to safely handle database name
            cursor.execute(sql.SQL("CREATE DATABASE {}").format(
                sql.Identifier(self.test_db)
            ))
            
            cursor.close()
            conn.close()
            
            self.print_success(f"Created database '{self.test_db}'")
            return True
        except psycopg2.Error as e:
            self.print_error(f"Failed to create database '{self.test_db}': {e}")
            return False
            
    def drop_test_database(self) -> bool:
        """Drop the test database if it exists"""
        self.print_status(f"Dropping test database '{self.test_db}'...")
        
        if not self.database_exists(self.test_db):
            self.print_warning(f"Database '{self.test_db}' does not exist")
            return True
            
        try:
            conn = psycopg2.connect(**self.get_connection_params())
            conn.set_isolation_level(psycopg2.extensions.ISOLATION_LEVEL_AUTOCOMMIT)
            cursor = conn.cursor()
            
            # Terminate existing connections to the database
            cursor.execute("""
                SELECT pg_terminate_backend(pid)
                FROM pg_stat_activity
                WHERE datname = %s AND pid <> pg_backend_pid()
            """, (self.test_db,))
            
            # Drop database
            cursor.execute(sql.SQL("DROP DATABASE IF EXISTS {}").format(
                sql.Identifier(self.test_db)
            ))
            
            cursor.close()
            conn.close()
            
            self.print_success(f"Dropped database '{self.test_db}'")
            return True
        except psycopg2.Error as e:
            self.print_error(f"Failed to drop database '{self.test_db}': {e}")
            return False
            
    def setup_schema(self) -> bool:
        """Setup the database schema for testing"""
        self.print_status(f"Setting up database schema for '{self.test_db}'...")
        
        schema_sql = """
            -- Drop existing table if it exists
            DROP TABLE IF EXISTS notes;
            
            -- Create notes table
            CREATE TABLE notes (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            
            -- Create indexes for better performance
            CREATE INDEX idx_notes_created_at ON notes(created_at);
            CREATE INDEX idx_notes_updated_at ON notes(updated_at);
        """
        
        try:
            conn = psycopg2.connect(**self.get_connection_params(self.test_db))
            cursor = conn.cursor()
            
            cursor.execute(schema_sql)
            conn.commit()
            
            cursor.close()
            conn.close()
            
            self.print_success("Database schema set up successfully")
            return True
        except psycopg2.Error as e:
            self.print_error(f"Failed to set up database schema: {e}")
            return False
            
    def verify_setup(self) -> bool:
        """Verify the test database setup by running a test query"""
        self.print_status("Verifying test database setup...")
        
        test_queries = [
            "INSERT INTO notes (title, content, created_at, updated_at) VALUES ('Test Note', 'Test Content', NOW(), NOW())",
            "SELECT COUNT(*) FROM notes WHERE title = 'Test Note'",
            "DELETE FROM notes WHERE title = 'Test Note'"
        ]
        
        try:
            conn = psycopg2.connect(**self.get_connection_params(self.test_db))
            cursor = conn.cursor()
            
            # Execute test queries
            for query in test_queries:
                cursor.execute(query)
                if "SELECT COUNT" in query:
                    result = cursor.fetchone()
                    if not result or result[0] != 1:
                        raise Exception("Unexpected count result")
                        
            conn.commit()
            cursor.close()
            conn.close()
            
            self.print_success("Database verification passed")
            return True
        except (psycopg2.Error, Exception) as e:
            self.print_error(f"Database verification failed: {e}")
            return False
            
    def show_environment(self) -> None:
        """Display current environment configuration"""
        self.print_status("Test Environment Configuration:")
        print(f"  Database Host: {self.db_host}")
        print(f"  Database Port: {self.db_port}")
        print(f"  Database User: {self.db_user}")
        print(f"  Test Database: {self.test_db}")
        print(f"  Connection URL: postgres://{self.db_user}:***@{self.db_host}:{self.db_port}/{self.test_db}")
        print()
        
    def run(self, verify_only: bool = False, clean_setup: bool = False) -> int:
        """Main execution flow"""
        print("🚀 Notes Server Integration Test Harness Setup")
        print("==============================================")
        print()
        
        self.show_environment()
        
        # Check if PostgreSQL is running
        if not self.check_postgres():
            if verify_only:
                self.print_error("PostgreSQL is not running. Cannot verify setup.")
                return 1
                
            self.print_status("Attempting to start PostgreSQL...")
            if not self.start_postgres():
                self.print_error("Setup failed. Please start PostgreSQL manually.")
                print("\nTo start PostgreSQL:")
                print("  • macOS (Homebrew): brew services start postgresql")
                print("  • Ubuntu/Debian: sudo systemctl start postgresql")
                print("  • CentOS/RHEL: sudo systemctl start postgresql")
                return 1
                
            # Wait for PostgreSQL to fully start
            time.sleep(2)
            
            # Verify it's running now
            if not self.check_postgres():
                self.print_error("PostgreSQL still not responding after start attempt")
                return 1
                
        if verify_only:
            self.print_status("Verification mode - checking existing setup...")
            if self.verify_setup():
                self.print_success("✅ Integration test harness is ready!")
                return 0
            else:
                self.print_error("❌ Integration test harness verification failed")
                return 1
                
        # Clean setup if requested
        if clean_setup:
            self.print_status("Clean setup requested - dropping existing database...")
            if not self.drop_test_database():
                return 1
                
        # Create test database
        if not self.create_test_database():
            return 1
            
        # Setup schema  
        if not self.setup_schema():
            return 1
            
        # Verify setup
        if not self.verify_setup():
            return 1
            
        print()
        self.print_success("✅ Integration test harness setup complete!")
        print()
        self.print_status("You can now run integration tests with:")
        print("  cargo test --test handlers_integration_tests")
        print()
        self.print_status("To run only unit tests (no database required):")
        print("  cargo test --lib")
        print()
        self.print_status("Environment variable for custom database URL:")
        print(f'  export TEST_DATABASE_URL="postgres://{self.db_user}:{self.db_password}@{self.db_host}:{self.db_port}/{self.test_db}"')
        
        return 0


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="Integration Test Harness Setup Script",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Environment Variables:
  DB_HOST         Database host (default: localhost)
  DB_PORT         Database port (default: 5432)
  DB_USER         Database user (default: postgres)
  DB_PASSWORD     Database password (default: password)
  TEST_DB         Test database name (default: notes_test)

Examples:
  python3 setup-integration-tests.py                     # Standard setup
  python3 setup-integration-tests.py --verify-only       # Only verify setup
  DB_PASSWORD=mypass python3 setup-integration-tests.py  # Custom password
  TEST_DB=my_test_db python3 setup-integration-tests.py --clean  # Custom DB with clean setup
        """
    )
    
    parser.add_argument(
        '--verify-only',
        action='store_true',
        help='Only verify existing setup without creating'
    )
    
    parser.add_argument(
        '--clean',
        action='store_true',
        help='Drop and recreate the test database'
    )
    
    args = parser.parse_args()
    
    setup = TestHarnessSetup()
    return setup.run(verify_only=args.verify_only, clean_setup=args.clean)


if __name__ == '__main__':
    sys.exit(main())