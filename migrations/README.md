# Database Migrations with Atlas

This directory contains database schema migrations managed by [Atlas](https://atlasgo.io/).

## Structure

```
migrations/
├── atlas.hcl              # Atlas configuration file
├── schema.hcl             # Atlas schema definition (HCL format)
├── migrations.go          # Go package to run migrations from the binary
├── atlas/
│   └── migrations/
│       ├── 20240101000001_initial_schema.sql  # Initial migration
│       └── atlas.sum                          # Migration checksum file
```

## How It Works

1. **Schema Definition**: The `schema.hcl` file defines the desired database schema using Atlas's HCL format
2. **Migration Files**: SQL migration files are stored in `atlas/migrations/`
3. **Embedded Migrations**: The Go binary embeds all migration files and runs them at startup
4. **Atlas CLI**: The Docker image includes the Atlas CLI for advanced migration operations

## Running Migrations

### Automatic (Production)
Migrations run automatically when the application starts:

```bash
./assoc-admin-bot
```

### Manual Migration Check
Run migrations only (without starting the app):

```bash
./assoc-admin-bot --migrate
```

### Using Atlas CLI (Advanced)

#### Generate new migration from schema changes:
```bash
cd migrations
atlas migrate diff --env local add_new_table
```

#### Apply migrations:
```bash
atlas migrate apply --url "sqlite:///path/to/assoc.db?_fk=1" --dir "file://atlas/migrations"
```

#### Check migration status:
```bash
atlas migrate status --url "sqlite:///path/to/assoc.db?_fk=1" --dir "file://atlas/migrations"
```

## Docker

The Dockerfile:
1. Installs Atlas CLI in the build stage
2. Copies Atlas CLI to the production image
3. Uses `docker-entrypoint.sh` to run migrations before starting the app

## Creating New Migrations

### Option 1: Manual SQL Migration

Create a new SQL file in `migrations/atlas/migrations/` following the naming convention:
```
YYYYMMDDHHMMSS_description.sql
```

Example: `20240317120000_add_user_preferences.sql`

Then update `atlas.sum`:
```bash
cd migrations/atlas/migrations && atlas migrate hash
```

### Option 2: Using Atlas Schema Diff (Recommended)

1. Update `schema.hcl` with your changes
2. Generate migration:
   ```bash
   cd migrations
   atlas migrate diff --env local your_change_description
   ```

## Environment Variables

- `DATA_PATH`: Path to the data directory (default: `/data`)
- `DATABASE_URL`: Full database URL for Atlas CLI operations

## Tables

- `config` - Application configuration
- `festAttendee` - Festival attendees
- `shopSell` - Shop items
- `userHistory` - User conversation history
- `historyPointers` - Tracks latest history slot per user
- `associates` - Associate information
- `treasuryUpdates` - Treasury update tracking
