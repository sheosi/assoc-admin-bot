//! Count how many fest attendees are coming

use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {
    #[description("El lugar al que chequear los asistentes")]
    lugar: AttendeePlace,
}

#[derive(serde::Deserialize, ToolParameters)]
enum AttendeePlace {
    #[serde(rename = "sundal")]
    Sundal,
    #[serde(rename = "bubu")]
    Bubu,
    #[serde(rename = "almuerzo")]
    Almuerzo,
}

impl AttendeePlace {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttendeePlace::Sundal => "Sundal",
            AttendeePlace::Bubu => "Bubu",
            AttendeePlace::Almuerzo => "Almuerzo",
        }
    }
}

/// Count how many are coming
pub struct CuantosVienen;

impl Tool for CuantosVienen {
    fn name(&self) -> &'static str {
        "Cuantos-Vienen"
    }

    fn description(&self) -> &'static str {
        "Cuántos vienen al evento que he pedido"
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
        let args: CallArgs = serde_json::from_str(arguments).map_err(|e| {
            anyhow::anyhow!(
                "No he podido leer los argumentos, coméntaselo a Cobi: {}",
                e
            )
        })?;

        let how_many: i64;
        match args.lugar {
            AttendeePlace::Bubu => {
                how_many = ctx.how_many_go_to_bubu().await?;
            }
            AttendeePlace::Sundal => {
                how_many = ctx.how_many_go_to_sundal().await?;
            }
            AttendeePlace::Almuerzo => {
                how_many = ctx.how_many_go_to_almuerzo().await?;
            }
        }

        Ok(ToolCallAction::Message(format!(
            "Vienen {} al {:?}",
            how_many,
            args.lugar.as_str()
        )))
    }
}
