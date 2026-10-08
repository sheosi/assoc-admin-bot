# Atlas configuration for SQLite
# https://atlasgo.io/atlas-schema/projects

# Define environment for local development and production
env {
  name = atlas.env
  # SQLite URL - will be set via environment variable
  url = getenv("DATABASE_URL")
  
  # Schema file location
  src = "file://schema.hcl"
  
  # Migration directory
  migration {
    dir = "file://migrations"
  }
}
