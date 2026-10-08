# Assoc admin Bot - Telegram Bot for Association Management

A rust-based  agentic admin bot for a small association.

## Overview

This is a Telegram bot for managing association activities including:
- **AI Chat**: OpenAI GPT-4 integration with function calling
- **Fest Management**: Track attendees, payments, dietary requirements
- **Treasury**: Record and report financial updates
- **Assembly Management**: Google Calendar integration for reminders
- **Browser Automation**: Automatic payment checking via Selenium
- **Email Notifications**: SMTP integration for group emails

## Architecture

```
assoc-admin-bot/
├── Cargo.toml           # Rust dependencies
├── .env.example         # Environment template
├── src/
│   ├── main.rs         # Entry point
│   ├── models.rs       # Shared types and structs
│   ├── db.rs           # SQL queries (sqlx equivalent of sqlc)
│   ├── api/            # Axum HTTP handlers
│   ├── bot/            # Telegram bot logic (teloxide)
│   ├── services/       # Business logic
│   │   ├── association.rs   # Core association service
│   │   ├── browser.rs       # Payment checking
│   │   ├── calendar.rs      # Google Calendar
│   │   ├── email.rs         # SMTP client
│   │   └── infisical.rs     # Secrets management
│   └── tools/          # AI function tools
│       ├── association.rs
│       └── fest.rs
└── queries/            # SQL files (for sqlx compile-time checking)
```

## Key Technologies

- **axum**: Web framework for HTTP API
- **teloxide**: Telegram bot framework
- **sqlx**: Compile-time checked SQL (equivalent to sqlc)
- **async-openai**: OpenAI API client
- **thirtyfour**: Selenium WebDriver for browser automation
- **lettre**: SMTP email client
- **google-calendar3**: Google Calendar API

## Setup

### Prerequisites

- Rust 1.75+
- SQLite
- ChromeDriver (for payment checking)

### Installation

1. Clone and enter the directory:
```bash
cd assoc-admin-bot
```

2. Copy environment template:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Build the project:
```bash
cargo build --release
```

4. Run database migrations:
```bash
cargo run -- --migrate
```

5. Run the bot:
```bash
cargo run
```

## Configuration

Required environment variables:
- `TELEGRAM_TOKEN` - Telegram bot token from @BotFather
- `OPENAI_API_KEY` - OpenAI API key
- `SMTP_*` - Email configuration for notifications

Optional:
- `RURALVIA_USER/PASS` - For automatic payment checking
- `GOOGLE_CALENDAR_CREDENTIALS` - For assembly reminders
- `INFISICAL_*` - For external secrets management

## Database Schema

The database uses SQLite with the following tables:
- `config` - Bot configuration (admins, chat IDs)
- `festAttendee` - Fest attendees with payment/diet info
- `userHistory` - Chat history for AI context
- `treasuryUpdates` - Financial records
- `associates` - Association members with emails

## API Endpoints

- `GET /health` - Health check
- `POST /webhook/new-attendee` - Webhook for new fest attendee registration

## Migration from Go

This Rust version replaces:
- `sqlc` → `sqlx` (compile-time SQL checking)
- Standard library `net/http` → `axum`
- `go-telegram-bot-api` → `teloxide`
- Official OpenAI SDK → `async-openai`
- Selenium Go bindings → `thirtyfour`

## Development

### Database Queries

SQL queries are defined in `src/db.rs` using sqlx macros. Unlike sqlc, sqlx checks queries at compile time against a running database or using `sqlx-data.json`.

### Adding New Tools

Tools are defined in `src/tools/` and implement the `Tool` trait:
```rust

impl Tool for MyTool {
    fn name(&self) -> &'static str { "my_tool" }
    fn description(&self) -> &'static str { "What this tool does" }
    fn parameters(&self) -> serde_json::Value { /* JSON schema */ }
    
    async fn tool_call(&self, ctx: &mut AssociationContext, chat_id: ChatId, arguments: &str) 
        -> anyhow::Result<()> { /* Implementation */ }
}
```

## License

MIT
