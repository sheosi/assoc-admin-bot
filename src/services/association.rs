//! Association service - Core business logic for the Telegram bot and AI integration

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, Locale, Utc};
use deadpool_sqlite::rusqlite::Connection;
use deadpool_sqlite::Config;
use deadpool_sqlite::Runtime;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message as LettreMessage, SmtpTransport, Transport};
use teloxide::prelude::*;
use teloxide::types::{FileId, MessageId};
use tokio::sync::{Mutex, RwLock};

use crate::bot;
use crate::db::init_database;
use crate::models::{AppConfig, EmailConfig, RunningConf};
use crate::queries::{self, GetAssociateRow, WhoGoesToSundalRow};
use crate::tools::{new_tools_dict, ToolsTrait};
use crate::utils::get_assoc_env;
use botframework::ai::{AiService, GroqProvider};
use botframework::telegram::{HistoryStore, TgBot};
use botframework::utils::db_action;

const AI_MSG_LIMIT: usize = 50;

/// Context for association operations
pub struct AssociationContext {
    pub bot: Arc<TgBot>,
    pub db: deadpool_sqlite::Pool,
    pub running_conf: Arc<Mutex<RunningConf>>,
    pub email_client: Option<SmtpTransport>,
    pub ai_service: AiService<GroqProvider>,
    pub data_path: PathBuf,
    pub ai_msg_count: Arc<Mutex<usize>>,
    pub tool_call_fills: Arc<Mutex<HashMap<i64, Box<dyn FnMut(String) + Send>>>>,
    action_map: HashMap<String, Arc<ToolsTrait>>,
    email_address: Option<Mailbox>,
}

impl botframework::telegram::SimpleBotDispatch<GroqProvider> for AssociationContext {
    async fn process_message(&mut self, msg: botframework::telegram::Message) -> Result<()> {
        bot::process_message(self, msg).await
    }

    async fn handle_callback(
        &mut self,
        data: &str,
        tool: &str,
        query_msg: teloxide::types::MaybeInaccessibleMessage,
    ) -> Result<()> {
        bot::handle_callback(self, data, tool, query_msg).await
    }

    async fn is_allowed(&self, username: &str) -> Result<bool> {
        Ok(self.is_admin(username).await)
    }

    fn get_ai_service(&self) -> &AiService<GroqProvider> {
        &self.ai_service
    }

    fn get_bot(&self) -> &Arc<TgBot> {
        &self.bot
    }
}

struct AssocEnvData {
    tg_key: String,
    def_admin: String,
    ai_data: String,
}

fn load_data(prefix: &str, secrets: &mut HashMap<String, String>) -> AssocEnvData {
    // We want env var to have priority, but since envconfig overrides everything
    // we are going to populate secrets from infisical after the fact

    let prefix = prefix.to_ascii_uppercase();
    AssocEnvData {
        tg_key: if let Some(tg_key) = secrets.remove(&format!("{}_TGKEY", prefix)) {
            tg_key
        } else {
            get_assoc_env(&prefix, "TGKEY")
        },
        def_admin: get_assoc_env(&prefix, "DEFADMIN"),
        ai_data: get_assoc_env(&prefix, "AIDATA"),
    }
}

pub struct SimpleAttachment {
    pub name: String,
    pub content: String,
}

impl AssociationContext {
    /// Create a new association context
    pub async fn new(
        asso_name: &str,
        config: &AppConfig,
        secrets: &mut HashMap<String, String>,
    ) -> Result<Arc<RwLock<Self>>> {
        println!("code: {}", config.groq_key);
        fn time_string_now(out: &mut String) {
            let dt = chrono::Local::now(); // Or your specific date

            // %a = abbreviated weekday (sáb.)
            // %e = day of month (17)
            // %B = full month name (mayo)
            let format_str = "%a %e %B";

            if let Err(e) = dt.format_localized(format_str, Locale::es_ES).write_to(out) {
                tracing::error!("Failed to write format {:?}", e);
            }
        }

        let tools = new_tools_dict();

        // Open database connection (synchronous, but fast)
        let data_path = config.data_path.join(asso_name);
        let db_path = data_path.join("main.db");

        let env_data = load_data(asso_name, secrets);

        if !data_path.exists() {
            tokio::fs::create_dir_all(&data_path).await?;
        }

        let db = Config::new(db_path).create_pool(Runtime::Tokio1)?;
        db_action(&db, move |c| init_database(c)).await?;

        let bot = TgBot::new(env_data.tg_key).await;

        // Load admins from database
        let admins_json = db_action(&db, move |c| {
            Ok(queries::GetAdmins {}
                .query_opt(c)
                .context("Failed to load admins")?
                .map(|q| q.admins)
                .unwrap_or_else(|| "[]".to_string()))
        })
        .await?;

        let admins: Vec<String> = serde_json::from_str(&admins_json).unwrap_or_default();

        let mut running_conf = RunningConf::new();
        if admins.is_empty() {
            running_conf.add_admin(env_data.def_admin);
            // TODO: Add admin to db
        } else {
            for admin in admins {
                running_conf.add_admin(admin);
            }
        }

        let email_conf = EmailConfig::from_secrets(asso_name, secrets);

        // Setup email client
        let email_client = email_conf
            .as_ref()
            .and_then(|c| Self::create_email_client(c).ok());

        // Setup AI service
        let mut own_prompt = "Tienes la capacidad de recordar los últimos 5 mensajes. Tus respuestas son enviadas a través de un bot de Telegram, no uses encabezados, en vez de eso usa negrita y separación, no uses tablas, en vez de eso usa listas con puntos, para las listas usa el cáracter •.".to_string();
        own_prompt.push_str("Hoy es ");
        time_string_now(&mut own_prompt);
        own_prompt.push_str(". No hFables de ningún evento anterior a hoy");
        own_prompt.push_str(&env_data.ai_data);
        let ai_service = AiService::new(&config.groq_key, tools.ai_descr, own_prompt);

        Ok(Arc::new(RwLock::new(Self {
            bot,
            running_conf: Arc::new(Mutex::new(running_conf)),
            email_client,
            ai_service,
            data_path,
            db,
            ai_msg_count: Arc::new(Mutex::new(0)),
            tool_call_fills: Arc::new(Mutex::new(HashMap::new())),
            action_map: tools.dict,
            email_address: email_conf
                .map(|c| Mailbox::new(None, Address::from_str(&c.from_address).expect(""))),
        })))
    }

    pub async fn on_db<F, R>(&self, action: F) -> Result<R, anyhow::Error>
    where
        F: FnOnce(&mut Connection) -> Result<R, anyhow::Error> + Send + 'static,
        R: Send + 'static,
    {
        db_action(&self.db, action).await
    }

    fn create_email_client(email_config: &EmailConfig) -> Result<SmtpTransport> {
        let creds = Credentials::new(email_config.username.clone(), email_config.password.clone());

        let mailer = SmtpTransport::starttls_relay(&email_config.smtp_server)?
            .port(email_config.smtp_port)
            .credentials(creds)
            .build();

        Ok(mailer)
    }

    /// Check if user is admin
    pub async fn is_admin(&self, username: &str) -> bool {
        let conf = self.running_conf.lock().await;
        conf.is_admin(username)
    }

    /// Add admin
    pub async fn add_admin(&self, username: &str) -> Result<()> {
        // Update database
        let username = username.to_string();
        db_action(&self.db, move |c| {
            queries::AddAdmin::builder()
                .json_set(1.0) // Use index as f64 for JSON array append
                .build()
                .execute(c)
                .context("action")
        })
        .await?;

        // Update runtime config
        let mut conf = self.running_conf.lock().await;
        conf.add_admin(username);
        Ok(())
    }

    /// Remove admin
    pub async fn remove_admin(&self, username: &str) -> Result<()> {
        let current_admins: Vec<String> = {
            let conf = self.running_conf.lock().await;
            conf.admins.iter().cloned().collect()
        };

        let new_admins: Vec<String> = current_admins
            .into_iter()
            .filter(|a| a != username)
            .collect();

        let admins_json = serde_json::to_string(&new_admins)?;

        // Update database

        self.on_db(move |c| {
            queries::SetAdmins::builder()
                .admins(&admins_json)
                .build()
                .execute(c)?;
            Ok::<_, anyhow::Error>(())
        })
        .await?;

        let mut conf = self.running_conf.lock().await;
        conf.remove_admin(username);
        Ok(())
    }

    /// List admins
    pub async fn get_admins(&self) -> Vec<String> {
        let conf = self.running_conf.lock().await;
        conf.admins.iter().cloned().collect()
    }

    /// Check if AI message limit is reached
    pub async fn ai_msg_limit_reached(&self) -> bool {
        let count = self.ai_msg_count.lock().await;
        *count >= AI_MSG_LIMIT
    }

    /// Increment AI message count
    pub async fn count_ai_msg(&self) {
        let mut count = self.ai_msg_count.lock().await;
        *count += 1;
    }

    /// Send raw message
    pub async fn send_raw(&self, chat_id: ChatId, text: &str) -> Result<MessageId> {
        self.bot.send_raw(chat_id, text).await
    }

    /// Send markdown message
    pub async fn send_md(&self, chat_id: ChatId, text: &str) -> Result<MessageId> {
        self.bot.send_md(chat_id, text).await
    }

    /// Replace a message with confirmation
    pub async fn replace_confirm(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
        text: &str,
    ) -> Result<()> {
        self.bot.replace_confirm(chat_id, message_id, text).await
    }

    /// Send custom list with inline keyboard
    pub async fn send_custom_list(
        &self,
        chat_id: ChatId,
        text: String,
        items: Vec<String>,
    ) -> Result<MessageId> {
        self.bot.send_custom_list(chat_id, text, items).await
    }

    /// Replace message with a new list
    pub async fn replace_list(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
        text: &str,
    ) -> Result<()> {
        self.bot.replace_list(chat_id, message_id, text).await
    }

    /// Download file from Telegram
    pub async fn get_file(&self, file_id: FileId, filename: &str) -> Result<PathBuf> {
        let filepath = self.data_path.join("downloads").join(filename);
        self.bot.get_file(file_id, &filepath).await?;

        Ok(filepath)
    }

    /// Transcribe audio using OpenAI Whisper
    pub async fn transcribe_audio(&self, audio_path: &std::path::Path) -> Result<String> {
        // Run transcription in a blocking task to isolate the non-Send future
        let response = self.ai_service.transcribe_audio(audio_path).await;
        tokio::fs::remove_file(audio_path).await?;
        response
    }

    /// Send email to a group of recipients
    pub async fn send_group_mail(
        &self,
        recipients: Vec<String>,
        subject: &str,
        body: &str,
        attachment: Option<SimpleAttachment>,
    ) -> Result<()> {
        if let Some(email_address) = self.email_address.clone() {
            if let Some(ref client) = self.email_client {
                fn attach_ics_file(name: String, content: String) -> SinglePart {
                    let content_type = ContentType::parse("text/calendar").expect("");
                    Attachment::new(name).body(content, content_type)
                }

                let attachment = attachment.map(|a| attach_ics_file(a.name, a.content));

                for recipient in recipients {
                    let to: Mailbox = recipient.parse()?;

                    let email_builder = LettreMessage::builder()
                        .from(email_address.clone())
                        .to(to)
                        .subject(subject);

                    let email = if let Some(attachment) = attachment.clone() {
                        email_builder.multipart(
                            MultiPart::mixed()
                                .singlepart(
                                    SinglePart::builder()
                                        .header(ContentType::TEXT_PLAIN)
                                        .body(body.to_string()),
                                )
                                .singlepart(attachment),
                        )?
                    } else {
                        email_builder.body(body.to_string())?
                    };

                    client.send(&email)?;
                }
            }
        }

        Ok(())
    }

    /// Get bot user info
    pub fn get_bot_username(&self) -> &str {
        self.bot.get_bot_username()
    }

    pub fn get_bot_id(&self) -> UserId {
        self.bot.get_bot_id()
    }

    /// Get all associate emails
    pub async fn get_all_associate_emails(&self) -> Result<Vec<String>> {
        self.on_db(|conn| {
            let rows = queries::GetAllAssociateEmails {}.query_many(conn)?;
            let emails: Vec<String> = rows.into_iter().map(|r| r.email).collect();
            Ok(emails)
        })
        .await
    }

    /// Get treasury notifications chat ID
    pub async fn get_treasury_notifications_chat_id(&self) -> Result<Option<i64>> {
        self.on_db(|conn| {
            let result = queries::GetTreasuryNotificationsChatId {}.query_opt(conn)?;
            Ok(result.map(|r| r.treasury_notifications_chat_id).flatten())
        })
        .await
    }

    /// Set treasury notifications chat ID
    pub async fn set_treasury_notifications_chat_id(&self, chat_id: ChatId) -> Result<()> {
        self.on_db(move |conn| {
            queries::SetTreasuryNotificationsChatId::builder()
                .treasury_notifications_chat_id(Some(chat_id.0))
                .build()
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    /// Get assembly minutes chat ID
    pub async fn get_assembly_minutes_chat_id(&self) -> Result<Option<i64>> {
        self.on_db(|conn| {
            let result = queries::GetAssemblyMinutesChatId {}.query_opt(conn)?;
            Ok(result.map(|r| r.assembly_minutes_chat_id).flatten())
        })
        .await
    }

    /// Set assembly minutes chat ID
    pub async fn set_assembly_minutes_chat_id(&self, chat_id: ChatId) -> Result<()> {
        self.on_db(move |conn| {
            queries::SetAssemblyMinutesChatId::builder()
                .assembly_minutes_chat_id(Some(chat_id.0))
                .build()
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    /// Add a treasury update
    pub async fn add_treasury_update(&self, description: &str, amount: f64) -> Result<i64> {
        let description = description.to_string();
        self.on_db(move |conn| {
            let result = queries::AddTreasuryUpdate::builder()
                .description(&description)
                .amount(amount)
                .createdat(Utc::now().timestamp() as f64)
                .build()
                .query_one(conn)?;
            Ok(result.id)
        })
        .await
    }

    /// Get treasury updates in a date range
    pub async fn get_treasury_updates_in_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<queries::GetTreasuryUpdatesInPeriodRow>> {
        self.on_db(move |conn| {
            let rows = queries::GetTreasuryUpdatesInPeriod::builder()
                .treasuryupdates_createdat_1(start.timestamp() as f64)
                .treasuryupdates_createdat_2(end.timestamp() as f64)
                .build()
                .query_many(conn)?;
            Ok(rows)
        })
        .await
    }

    /// Add a new associate
    pub async fn add_associate(&self, nickname: &str, email: &str) -> Result<Option<String>> {
        let nickname = nickname.to_string();
        let email = email.to_string();
        self.on_db(move |conn| {
            let result = queries::AddAssociate::builder()
                .nickname(&nickname)
                .email(&email)
                .build()
                .query_opt(conn)?;
            Ok(result.map(|r| r.nickname))
        })
        .await
    }

    /// Remove an associate
    pub async fn remove_associate(&self, nickname: &str) -> Result<()> {
        let nickname = nickname.to_string();
        self.on_db(move |conn| {
            queries::RemoveAssociate::builder()
                .nickname(&nickname)
                .build()
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    /// List all associates
    pub async fn list_associates(&self) -> Result<Vec<queries::ListAssociatesRow>> {
        self.on_db(|conn| {
            let rows = queries::ListAssociates {}.query_many(conn)?;
            Ok(rows)
        })
        .await
    }

    /// Update an associate
    pub async fn update_associate(
        &self,
        nickname: &str,
        email: &str,
        old_nickname: &str,
    ) -> Result<()> {
        let nickname = nickname.to_string();
        let email = email.to_string();
        let old_nickname = old_nickname.to_string();
        self.on_db(move |conn| {
            queries::UpdateAssociate::builder()
                .associates_nickname_1(&nickname)
                .email(&email)
                .associates_nickname_2(&old_nickname)
                .build()
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    /// Get fest attendees
    pub async fn get_fest_attendees(&self) -> Result<Vec<queries::GetFestAttendeesRow>> {
        self.on_db(|conn| {
            let rows = queries::GetFestAttendees {}.query_many(conn)?;
            Ok(rows)
        })
        .await
    }

    pub async fn get_fest_attendees_names(&self) -> Result<Vec<queries::GetAttendessNamesRow>> {
        self.on_db(|c| {
            let rows = queries::GetAttendessNames {}.query_many(c)?;
            Ok(rows)
        })
        .await
    }

    /// Check if an attendee has paid for fest
    pub async fn has_paid_fest(&self, code_name: &str) -> Result<Option<i64>> {
        let code_name = code_name.to_string();
        self.on_db(move |conn| {
            let result = queries::HasPaidFest::builder()
                .codename(&code_name)
                .build()
                .query_opt(conn)?;
            Ok(result.map(|r| r.haspaid))
        })
        .await
    }

    /// Set paid status by legal name
    pub async fn legal_name_set_paid(&self, legal_name: &str, has_paid: i64) -> Result<()> {
        let legal_name = legal_name.to_string();
        self.on_db(move |conn| {
            queries::LegalNameSetPaid::builder()
                .legalname(&legal_name)
                .haspaid(has_paid)
                .build()
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    /// Count how many go to sundal
    pub async fn how_many_go_to_sundal(&self) -> Result<i64> {
        self.on_db(|conn| {
            let result = queries::HowManyGoToSundal {}.query_one(conn)?;
            Ok(result.count)
        })
        .await
    }

    /// Count how many go to bubu
    pub async fn how_many_go_to_bubu(&self) -> Result<i64> {
        self.on_db(|conn| {
            let result = queries::HowManyGoToBubu {}.query_one(conn)?;
            Ok(result.count)
        })
        .await
    }

    /// Count how many go to almuerzo
    pub async fn how_many_go_to_almuerzo(&self) -> Result<i64> {
        self.on_db(|conn| {
            let result = queries::HowManyGoToAlmuerzo {}.query_one(conn)?;
            Ok(result.count)
        })
        .await
    }

    /// Count standard diet
    pub async fn how_many_standard_diet(&self) -> Result<i64> {
        self.on_db(|conn| {
            let result = queries::HowManyStandardDiet {}.query_one(conn)?;
            Ok(result.count)
        })
        .await
    }

    /// Get vegetarian attendees
    pub async fn who_vegetarian(&self) -> Result<Vec<String>> {
        self.on_db(|conn| {
            let rows = queries::WhoVegetarian {}.query_many(conn)?;
            let names: Vec<String> = rows.into_iter().map(|r| r.codename).collect();
            Ok(names)
        })
        .await
    }

    /// Get vegan attendees
    pub async fn who_vegan(&self) -> Result<Vec<String>> {
        self.on_db(|conn| {
            let rows = queries::WhoVegan {}.query_many(conn)?;
            let names: Vec<String> = rows.into_iter().map(|r| r.codename).collect();
            Ok(names)
        })
        .await
    }

    /// Get attendees with allergies
    pub async fn who_allergies(&self) -> Result<Vec<(String, Option<String>)>> {
        self.on_db(|conn| {
            let rows = queries::WhoAllergies {}.query_many(conn)?;
            let allergies: Vec<(String, Option<String>)> = rows
                .into_iter()
                .map(|r| (r.codename, r.allergies))
                .collect();
            Ok(allergies)
        })
        .await
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<ToolsTrait>> {
        self.action_map.get(name).cloned()
    }

    pub async fn get_associate(&self, nickname: &str) -> Result<Option<GetAssociateRow>> {
        let nickname = nickname.to_string();
        self.on_db(move |c| {
            let search = queries::GetAssociate::builder()
                .nickname(&nickname)
                .build()
                .query_opt(c)?;

            Ok(search)
        })
        .await
    }

    pub async fn who_goes_to_sundal(&self) -> Result<Vec<WhoGoesToSundalRow>> {
        self.on_db(move |c| {
            let search = queries::WhoGoesToSundal::builder().build().query_many(c)?;
            Ok(search)
        })
        .await
    }
}

impl HistoryStore for AssociationContext {
    /// Get chat history for a user
    async fn get_history(&self, user_id: ChatId) -> Result<Vec<(String, String)>> {
        let rows = self
            .on_db(move |c| {
                let history = queries::GetHistory::builder()
                    .userid(user_id.0)
                    .build()
                    .query_many(c)?;
                Ok::<_, anyhow::Error>(history)
            })
            .await?;

        Ok(rows.into_iter().map(|r| (r.message, r.answer)).collect())
    }

    async fn push_history(&self, chat_id: ChatId, text: String, answer: String) -> Result<()> {
        let _ = self
            .on_db(move |c| {
                let slot = match queries::RotatePointer::builder()
                    .userid(chat_id.0)
                    .build()
                    .query_one(c)
                {
                    Ok(s) => s.lastslot,
                    Err(_) => return Ok(()),
                };

                let _ = queries::SaveHistory::builder()
                    .userid(chat_id.0)
                    .slotid(slot)
                    .message(&text)
                    .answer(&answer)
                    .createdat(Utc::now().timestamp() as f64)
                    .build()
                    .execute(c);
                Ok::<_, anyhow::Error>(())
            })
            .await;

        Ok(())
    }
}
