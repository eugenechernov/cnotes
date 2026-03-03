# Notes Server

A high-performance REST API server built with Rust, Axum, and PostgreSQL for managing notes.

## Features

- **CRUD Operations**: Create, read, update, and delete notes
- **PostgreSQL Database**: Reliable data persistence with ACID transactions
- **Input Validation**: Comprehensive request validation
- **Structured Logging**: JSON logging with tracing
- **Auto Migrations**: Automatic database schema management
- **CORS Support**: Cross-origin resource sharing for web clients
- **Health Check**: Basic health check endpoint

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/api/notes` | Get all notes |
| GET | `/api/notes/:id` | Get note by ID |
| POST | `/api/notes` | Create new note |
| PUT | `/api/notes/:id` | Update note |
| DELETE | `/api/notes/:id` | Delete note |

## Quick Start

### Prerequisites

- Rust (latest stable)
- PostgreSQL 12+
- Docker (optional, for database)

### Development Setup

1. **Clone and navigate to server directory**
   ```bash
   cd server
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your database configuration
   ```

3. **Start PostgreSQL (using Docker)**
   ```bash
   docker run --name notes-postgres \
     -e POSTGRES_DB=notes \
     -e POSTGRES_USER=postgres \
     -e POSTGRES_PASSWORD=password \
     -p 5432:5432 \
     -d postgres:15
   ```

4. **Run the server**
   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000`

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | `postgresql://postgres:password@localhost:5432/notes` |
| `PORT` | Server port | `3000` |
| `HOST` | Server host | `0.0.0.0` |
| `RUST_LOG` | Logging level | `notes_server=debug,tower_http=debug,sqlx=info` |

## Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── config/              # Configuration management
│   ├── mod.rs
│   └── settings.rs
├── database/            # Database connection and utilities
│   ├── mod.rs
│   └── connection.rs
├── handlers/            # HTTP request handlers
│   ├── mod.rs
│   └── notes.rs
└── models/              # Data models and validation
    ├── mod.rs
    └── note.rs
```

## API Usage Examples

### Create a Note
```bash
curl -X POST http://localhost:3000/api/notes \
  -H "Content-Type: application/json" \
  -d '{"title": "My First Note", "content": "This is the note content"}'
```

### Get All Notes
```bash
curl http://localhost:3000/api/notes
```

### Get Note by ID
```bash
curl http://localhost:3000/api/notes/1
```

### Update a Note
```bash
curl -X PUT http://localhost:3000/api/notes/1 \
  -H "Content-Type: application/json" \
  -d '{"title": "Updated Title", "content": "Updated content"}'
```

### Delete a Note
```bash
curl -X DELETE http://localhost:3000/api/notes/1
```

## Response Format

### Success Response
```json
{
  "success": true,
  "data": {
    "note": {
      "id": 1,
      "title": "My Note",
      "content": "Note content",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  },
  "error": null
}
```

### Error Response
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "NOT_FOUND",
    "message": "Note with id 123 not found"
  }
}
```

## Development

### Running Tests
```bash
cargo test
```

### Database Migrations
Migrations are automatically applied on server startup. Migration files are in the `migrations/` directory.

### Checking Code Quality
```bash
cargo clippy
cargo fmt
```

## Production Deployment

### Using Docker
1. Build the Docker image
2. Set environment variables
3. Run with PostgreSQL database
4. Configure reverse proxy (nginx/Apache)
5. Set up SSL/TLS certificates

### Performance Tuning
- Adjust database connection pool settings
- Configure appropriate logging levels
- Use release build for production (`cargo build --release`)
- Monitor memory usage and database query performance

## Troubleshooting

### Common Issues

1. **Database connection failed**
   - Check PostgreSQL is running
   - Verify DATABASE_URL is correct
   - Ensure database exists

2. **Port already in use**
   - Change PORT in .env file
   - Kill existing process using the port

3. **Migration errors**
   - Check database permissions
   - Verify migration SQL syntax
   - Reset database if needed

### Logging

The server uses structured logging. Set `RUST_LOG` environment variable to control log levels:
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information
- `debug`: Detailed debugging info
- `trace`: Very verbose output

## License

This project is part of the Notes application suite.