use crate::services::association::{AssociationContext, SimpleAttachment};
use crate::tools::Tool;
use crate::utils;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};

use teloxide::types::ChatId;

/// Arguments for the create_assembly tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description("Fecha y hora de la asamblea en formato RFC3339 (ejemplo: 2024-03-15T18:00:00)")]
    date: &'a str,
    #[description("Additional description")]
    description: &'a str,
}

/// Create assembly tool
pub struct CreateAssembly;

impl Tool for CreateAssembly {
    fn name(&self) -> &'static str {
        "create_assembly"
    }

    fn description(&self) -> &'static str {
        "Create and schedule a new assembly"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        _chat_id: ChatId,
        arguments: &str,
    ) -> Result<ToolCallAction> {
        let args: CallArgs = serde_json::from_str(arguments)?;

        let assembly_date = match chrono::DateTime::parse_from_rfc3339(args.date) {
            Ok(t) => t,
            Err(_) => chrono::DateTime::parse_from_str(args.date, "%Y-%m-%dT%H:%M:%S")?,
        }
        .to_utc();

        let emails = ctx.get_all_associate_emails().await?;

        if emails.is_empty() {
            return Ok(
                "Lo siento, ha habido un problema al obtener los emails de los asociados".into(),
            );
        }

        let ics_data = utils::create_ics_file(assembly_date, args.description);
        let subject = format!(
            "Convocatoria de Asamblea - {}",
            assembly_date.format("%d/%m/%Y")
        );
        let assembly_body_date = assembly_date.format("%d/%m/%Y a las %H:%M");
        let summary = args.description;
        let body = format!("Hola,\n\nSe ha convocado una nueva asamblea.\n\nFECHA Y HORA: {assembly_body_date}\n\nRESUMEN:\n{summary}\n\nSe adjunta un archivo de calendario para que puedas añadir el evento a tu agenda.\n\nSaludos.");
        let emails_count = emails.len();
        ctx.send_group_mail(
            emails,
            &subject,
            &body,
            Some(SimpleAttachment {
                name: ics_data.0,
                content: ics_data.1,
            }),
        )
        .await?;

        let msg = format!(
            "✅ Asamblea creada para el {assembly_body_date} y convocatoria enviada a {emails_count} asociados",
        );

        Ok(msg.into())
    }
}
