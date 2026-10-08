//! Telegram bot module using teloxide
//! Handles incoming messages, voice memos, and callback queries

use anyhow::Result;
use async_openai::types::chat::ChatCompletionMessageToolCalls;
use teloxide::types::MaybeInaccessibleMessage;

use tracing::warn;

use crate::services::association::AssociationContext;
use crate::tools::Tool;
use botframework::telegram::Message as AppMessage;

/// Bot handler that processes Telegram updates

pub async fn handle_callback(
    context: &mut AssociationContext,
    tool: &str,
    data: &str,
    msg: MaybeInaccessibleMessage,
) -> Result<()> {
    let response = if let Some(tool) = context.get_tool(tool) {
        tool.handle_callback(context, data).await?
    } else {
        warn!("Tool not found for callback: {}", tool);
        "El tiempo de espera de la operación se ha terminado".to_string()
    };

    context
        .replace_confirm(msg.chat().id, msg.id(), &response)
        .await?;

    Ok(())
}

/// Process a message through AI and handle tool calls
pub async fn process_message(context: &mut AssociationContext, msg: AppMessage) -> Result<()> {
    let is_admin = context.is_admin(&msg.username).await;

    // Check AI message limit for non-admins
    if !is_admin && context.ai_msg_limit_reached().await {
        context
            .send_raw(
                msg.chat_id,
                "Has alcanzado el límite de mensajes de IA. Contacta con un administrador.",
            )
            .await?;
        return Ok(());
    }

    // Process through AI with mutable context

    // Process message through AI
    let ai_result = botframework::telegram::process_ai(
        context,
        &context.ai_service,
        &context.bot,
        msg.chat_id,
        msg.text.clone(),
        is_admin,
    )
    .await;

    match ai_result {
        Ok(Some(tool_call)) => {
            let ChatCompletionMessageToolCalls::Function(tool_call) =
                tool_call.into_iter().next().unwrap()
            else {
                return Ok(());
            };

            if !is_admin {
                context
                    .send_raw(
                        msg.chat_id,
                        "Estas características están limitadas a administradores.",
                    )
                    .await?;
                return Ok(());
            }

            let tool_call = tool_call.function;
            // Handle tool call

            let Some(tool) = context.get_tool(&tool_call.name) else {
                warn!("Unknown tool called: {}", &tool_call.name);
                if let Err(e) = context
                    .send_raw(msg.chat_id, "Disculpa, no reconozco esa herramienta.")
                    .await
                {
                    tracing::error!("Failed to send error msg: {e}")
                }
                return Ok(());
            };

            let tool_action = tool
                .tool_call(context, msg.chat_id, &tool_call.arguments)
                .await;

            botframework::telegram::perform_tool_action(
                tool_action,
                &tool_call.name,
                &context.bot,
                msg.chat_id,
            )
            .await;
        }
        Ok(None) => {
            // Message processed successfully, no tool call
        }
        Err(e) => {
            tracing::error!("AI processing error: {}", e);
            context
                .send_raw(msg.chat_id, "Error procesando el mensaje")
                .await?;
        }
    }
    Ok(())
}
