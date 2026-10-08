// Package migrations handles database schema migrations using Atlas
// The migration files are embedded into the binary at build time
package migrations

import (
	"context"
	"embed"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"

	"ariga.io/atlas-go-sdk/atlasexec"
)

//go:embed atlas/migrations/*.sql
//go:embed atlas/migrations/atlas.sum
var migrationFS embed.FS

//go:embed schema.hcl
var schemaFile []byte

// Runner handles database migrations
type Runner struct {
	dbPath string
}

// NewRunner creates a new migration runner
func NewRunner(dbPath string) *Runner {
	return &Runner{
		dbPath: dbPath,
	}
}

// RunMigrations executes all pending migrations using Atlas CLI
func (r *Runner) RunMigrations(ctx context.Context) error {
	log.Println("[INFO] Starting database migrations...")

	// Create a temporary directory to extract migration files
	tmpDir, err := os.MkdirTemp("", "atlas-migrations-*")
	if err != nil {
		return fmt.Errorf("failed to create temp dir: %w", err)
	}
	defer os.RemoveAll(tmpDir)

	// Extract migration files from embedded FS
	migrationsDir := filepath.Join(tmpDir, "migrations")
	if err := r.extractMigrations(migrationsDir); err != nil {
		return fmt.Errorf("failed to extract migrations: %w", err)
	}

	// Extract schema file
	schemaPath := filepath.Join(tmpDir, "schema.hcl")
	if err := os.WriteFile(schemaPath, schemaFile, 0644); err != nil {
		return fmt.Errorf("failed to write schema file: %w", err)
	}

	// Initialize Atlas client
	client, err := atlasexec.NewClient(tmpDir, "atlas")
	if err != nil {
		// If atlas CLI is not available, fall back to SQL execution
		log.Println("[WARN] Atlas CLI not found, falling back to direct SQL execution")
		return r.runMigrationsDirectly(ctx, migrationsDir)
	}

	// Convert to absolute path (Atlas requires absolute paths)
	absDbPath, err := filepath.Abs(r.dbPath)
	if err != nil {
		return fmt.Errorf("failed to get absolute database path: %w", err)
	}

	// Ensure database directory exists
	dbDir := filepath.Dir(absDbPath)
	if err := os.MkdirAll(dbDir, 0755); err != nil {
		return fmt.Errorf("failed to create database directory: %w", err)
	}

	// Ensure database file exists (Atlas requires it)
	if _, err := os.Stat(absDbPath); os.IsNotExist(err) {
		if f, err := os.Create(absDbPath); err != nil {
			return fmt.Errorf("failed to create database file: %w", err)
		} else {
			f.Close()
		}
	}

	// Set up migration parameters
	dbURL := fmt.Sprintf("sqlite://%s?_fk=1", absDbPath)

	// Apply migrations using Atlas
	res, err := client.MigrateApply(ctx, &atlasexec.MigrateApplyParams{
		URL:        dbURL,
		DirURL:     fmt.Sprintf("file://%s", migrationsDir),
		AllowDirty: true,
	})
	if err != nil {
		return fmt.Errorf("failed to apply migrations: %w", err)
	}

	log.Printf("[INFO] Applied %d migrations", len(res.Applied))
	return nil
}

func (r *Runner) extractMigrations(targetDir string) error {
	return fs.WalkDir(migrationFS, "atlas/migrations", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		// Create directories
		if d.IsDir() {
			return os.MkdirAll(filepath.Join(targetDir, filepath.Base(path)), 0755)
		}

		// Copy files
		data, err := migrationFS.ReadFile(path)
		if err != nil {
			return fmt.Errorf("failed to read embedded file %s: %w", path, err)
		}

		targetPath := filepath.Join(targetDir, filepath.Base(path))
		if err := os.WriteFile(targetPath, data, 0644); err != nil {
			return fmt.Errorf("failed to write file %s: %w", targetPath, err)
		}

		return nil
	})
}

// runMigrationsDirectly executes SQL migrations without Atlas CLI
func (r *Runner) runMigrationsDirectly(ctx context.Context, migrationsDir string) error {
	// This is a fallback when Atlas CLI is not available
	// We would need to parse SQL files and execute them manually
	// For now, just return an error to require Atlas CLI
	return fmt.Errorf("Atlas CLI is required for migrations. Please install it: https://atlasgo.io/getting-started")
}
