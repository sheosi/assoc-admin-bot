# Infisical Secrets Integration

This application supports fetching secrets from [Infisical](https://infisical.com/) during association setup. This allows you to manage sensitive configuration (API keys, passwords, tokens) securely in a centralized secrets management platform instead of local `.env` files.

## Overview

When the association starts up, the `loadEnv()` function:

1. Loads the local `.env` file (if present)
2. Checks if Infisical is configured via environment variables
3. Authenticates with Infisical using Universal Auth (Machine Identity)
4. Fetches all secrets from your Infisical project into a Go map (variables)
5. Populates configuration directly from the secrets map (taking precedence over environment variables)
6. Falls back to environment variables for any secrets not found in Infisical

Secrets from Infisical are stored in regular Go variables (a `map[string]string`), not environment variables.

## Configuration

To enable Infisical integration, set the following environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `INFISICAL_CLIENT_ID` | Yes | - | Machine Identity Client ID from Infisical |
| `INFISICAL_CLIENT_SECRET` | Yes | - | Machine Identity Client Secret from Infisical |
| `INFISICAL_PROJECT_ID` | Yes | - | Your Infisical project ID (required to load secrets) |
| `INFISICAL_ENVIRONMENT` | No | `dev` | Environment to fetch secrets from (e.g., `dev`, `staging`, `production`) |
| `INFISICAL_API_URL` | No | `https://app.infisical.com` | Self-hosted Infisical URL (if applicable) |

## Required Secrets

Store the following secrets in your Infisical project. They will be loaded into environment variables at startup:

| Secret Key | Purpose |
|------------|---------|
| `ASSOC_TGKEY` | Telegram Bot API key |
| `ASSOC_EMAIL_ADRESS` | SMTP email username |
| `ASSOC_EMAIL_PASSWORD` | SMTP email password |
| `ASSOC_BANK_USER` | Bank portal username |
| `ASSOC_BANK_PASS` | Bank portal password |
| `GROQ_API_KEY` | Groq/OpenAI API key for AI features |
| `DATA_PATH` | Path to data directory |
| `GOOGLE_CALENDAR_CREDENTIALS` | Path to Google Calendar credentials file |

The application uses the `envconfig` library to map environment variables (including those loaded from Infisical) to the configuration struct, so secrets must match the expected naming convention.

## Setup in Infisical

### 1. Create a Project

1. Log in to your Infisical dashboard
2. Create a new project (or use existing)
3. Note the Project ID from the project settings

### 2. Create a Machine Identity

1. Go to **Organization Settings** → **Machine Identities**
2. Create a new Machine Identity
3. Create **Universal Auth** credentials for the identity
4. Copy the **Client ID** and **Client Secret**

### 3. Add the Identity to Your Project

1. Go to your Project → **Access Control**
2. Add the Machine Identity with appropriate permissions (at minimum: read secrets)

### 4. Add Secrets

Add your secrets in the Infisical dashboard:
- Use the exact secret keys listed above
- Organize by environment (`dev`, `staging`, `production`)

## Docker Deployment

When running in Docker, pass Infisical credentials as environment variables:

```yaml
# docker-compose.yml
services:
  app:
    image: assoc-admin-bot:latest
    environment:
      - INFISICAL_CLIENT_ID=${INFISICAL_CLIENT_ID}
      - INFISICAL_CLIENT_SECRET=${INFISICAL_CLIENT_SECRET}
      - INFISICAL_PROJECT_ID=your-project-id
      - INFISICAL_ENVIRONMENT=production
      # Other non-secret env vars can still be set directly
      - DATA_PATH=/data
```

Or with `docker run`:

```bash
docker run \
  -e INFISICAL_CLIENT_ID=your-client-id \
  -e INFISICAL_CLIENT_SECRET=your-client-secret \
  -e INFISICAL_PROJECT_ID=your-project-id \
  -e INFISICAL_ENVIRONMENT=production \
  -v ./data:/data \
  assoc-admin-bot:latest
```

## How It Works

The `loadEnv()` function in `association.go` handles all Infisical integration:

1. **Load `.env` file**: First loads any local `.env` file if present
2. **Check Infisical config**: Looks for `INFISICAL_CLIENT_ID` and `INFISICAL_CLIENT_SECRET`
3. **Authenticate**: Creates an Infisical client and authenticates via Universal Auth
4. **Fetch secrets**: Fetches all secrets from the specified project/environment into a Go map
5. **Populate config directly**: Sets configuration values directly from the secrets map
6. **Fallback to env**: Uses `envconfig` to fill any missing values from environment variables

The `getSecretFromMapOrEnv()` helper function provides a unified way to look up secrets:
- First checks the Infisical secrets map
- Falls back to `os.Getenv()` if not found in the map

## Fallback Behavior

The application implements graceful fallback:

1. **If Infisical is not configured** (`INFISICAL_CLIENT_ID` or `INFISICAL_CLIENT_SECRET` missing): Skips Infisical and uses only local `.env` and system environment variables
2. **If `INFISICAL_PROJECT_ID` is not set**: Skips Infisical secret loading
3. **If Infisical authentication fails**: Logs a warning and continues with local configuration
4. **If secret loading fails**: Logs a warning and continues with local configuration

**Precedence order** (highest to lowest):
1. Infisical secrets (loaded into the secrets map)
2. Environment variables (from `.env` file or system)
3. Application defaults

## Security Considerations

- **Never commit** `INFISICAL_CLIENT_ID` or `INFISICAL_CLIENT_SECRET` to version control
- Use your platform's secret management for the Infisical credentials (e.g., Docker secrets, Kubernetes secrets, CI/CD environment variables)
- The client only reads secrets and never writes back to Infisical
- Secrets are stored in Go variables (memory) only; they're not persisted to disk or set as environment variables
- The secrets map is passed directly to `NewAssociation()` and used for lookups

## Troubleshooting

### Authentication Failures

```
[WARN] Failed to create Infisical client: failed to authenticate with Infisical: authentication failed: ...
```

- Verify your `INFISICAL_CLIENT_ID` and `INFISICAL_CLIENT_SECRET` are correct
- Ensure the Machine Identity has Universal Auth enabled
- Check that the Machine Identity has access to the project

### Secret Not Found

```
[WARN] Failed to load secrets from Infisical: failed to fetch secrets: ...
```

- Verify `INFISICAL_PROJECT_ID` is correct
- Check that the Machine Identity has read permissions on the project
- Ensure secrets exist in the specified environment

### Migration from .env

To migrate from local `.env` files to Infisical:

1. Create your Infisical project and add all secrets
2. Set `INFISICAL_CLIENT_ID`, `INFISICAL_CLIENT_SECRET`, and `INFISICAL_PROJECT_ID`
3. Remove sensitive values from `.env` (keep non-sensitive configuration)
4. The application will automatically prefer Infisical secrets when available

## Code Implementation

The Infisical integration is implemented in two files:

### `/infisical/infisical.go`
The Infisical client library providing:
- `NewClient()` - Creates and authenticates a client using Universal Auth
- `GetSecretsMap()` - Fetches all secrets and returns them as a `map[string]string`
- `LoadSecretsIntoEnv()` - (Deprecated) Fetches all secrets and sets them as environment variables
- Self-hosted Infisical instance support

### `/association/association.go`
The `loadEnv()` function orchestrates the integration:
- Loads `.env` file
- Calls `loadInfisicalSecrets()` to fetch secrets from Infisical into a map
- Populates `AssocEnvData` directly from the secrets map (taking precedence over env vars)
- Uses `envconfig` to fill any missing values from environment variables
- Returns the secrets map for runtime lookups (email, bank credentials)

The `getSecretFromMapOrEnv()` helper provides a unified lookup that prefers Infisical secrets but falls back to environment variables.
