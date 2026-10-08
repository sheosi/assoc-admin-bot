//! Calculate bocatas tool for AI function calling

use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// Arguments for the calculate_bocatas tool
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {}

/// Calculate bocatas (sandwiches) needed
pub struct CalcBocatas;

impl Tool for CalcBocatas {
    fn name(&self) -> &'static str {
        "calculate_bocatas"
    }

    fn description(&self) -> &'static str {
        "Calculate how many bocadillos (sandwiches) are needed for the fest"
    }

    fn parameters(&self) -> Properties {
        CallArgs::parameters()
    }

    async fn tool_call(
        &self,
        ctx: &mut AssociationContext,
        _chat_id: ChatId,
        _arguments: &str,
    ) -> Result<ToolCallAction> {
        // Get people going to almuerzo
        let total = ctx.how_many_go_to_almuerzo().await?;

        // Get dietary restrictions
        let vegetarians = ctx.who_vegetarian().await?;

        let vegans = ctx.who_vegan().await?;
        let standard = ctx.how_many_standard_diet().await?;
        let allergies = ctx.who_allergies().await?;

        let msg = format!(
            "*Total de asistentes al Almuerzo:* {total}\n\n\
            *Desglose:*\n\
            • Almuerzos normales: {standard} personas \n\
            • Vegetarianos - {}: {}\n\
            • Veganos - {}: {} \n\n\
            • Alergias: {}
            ",
            vegetarians.len(),
            vegetarians.join(", "),
            vegans.len(),
            vegans.join(", "),
            allergies.into_iter().fold(
                String::with_capacity(100),
                |mut msg, (attendee, allergy)| {
                    if let Some(allergy) = allergy {
                        msg.push_str(&format!("{}: {},", attendee, allergy));
                    }
                    msg
                }
            )
        );

        Ok(ToolCallAction::MarkDown(msg))
    }
}
