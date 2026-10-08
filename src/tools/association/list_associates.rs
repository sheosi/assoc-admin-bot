use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use teloxide::types::ChatId;

/// CallArgs for list_associates - no parameters required
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs {}

/// List associates tool
pub struct ListAssociates;

impl Tool for ListAssociates {
    fn name(&self) -> &'static str {
        "List-Associates"
    }

    fn description(&self) -> &'static str {
        "Lista todos los asociados registrados con su nombre en clave y correo electrónico"
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
        let associates = ctx.list_associates().await?;

        if associates.is_empty() {
            Ok(format!("No hay asociados registrados").into())
        } else {
            let mut msg = "*Associates:*\n\n".to_string();
            for (i, assoc) in associates.iter().enumerate() {
                msg.push_str(&format!("• {} - {}\n", assoc.nickname, assoc.email));

                if i < associates.len() {
                    msg.push_str("\n");
                }
            }

            Ok(ToolCallAction::MarkDown(msg))
        }
    }
}
