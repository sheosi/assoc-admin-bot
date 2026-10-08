use crate::services::association::AssociationContext;
use crate::tools::Tool;
use anyhow::Result;
use botframework::telegram::ToolParameters;
use botframework::telegram::{Properties, ToolCallAction};
use printpdf::{Mm, PdfSaveOptions, Pt, TextItem};
use std::path::PathBuf;
use teloxide::types::ChatId;

/// Arguments for the post_assembly_minutes tool
/// This struct is used both for deserialization and for generating the parameter schema
#[derive(serde::Deserialize, ToolParameters)]
struct CallArgs<'a> {
    #[description(
        "Fecha y hora de inicio de la asamblea en formato ISO 8601 (ejemplo: 2024-03-15T18:00:00)"
    )]
    fecha_hora: &'a str,

    #[description("Canal o medio a través del cual se realizó la asamblea (ejemplo: Telegram, Discord, Presencial)")]
    canal: &'a str,

    #[description("Hora de finalización de la asamblea en formato HH:MM (ejemplo: 20:30)")]
    hora_fin: &'a str,

    #[description("Votación para la aprobación del acta anterior")]
    punto1_aprobacion_acta: Punto1AprobacionActa,

    #[description("Informe del presidente sobre el estado de la asociación y actualizaciones")]
    punto2_informe_presidente: &'a str,

    #[description("Puntos tratados en el orden del día")]
    puntos_del_orden_dia: Vec<PuntoOrdenDia<'a>>,

    #[description("Contenido de la sección de ruegos y preguntas")]
    ruegos_y_preguntas: &'a str,
}

/// Voting results for approving the previous minutes
#[derive(serde::Deserialize, ToolParameters)]
struct Punto1AprobacionActa {
    #[description("Número de votos a favor")]
    a_favor: i64,

    #[description("Número de votos en contra")]
    en_contra: i64,

    #[description("Número de abstenciones")]
    abstenciones: i64,
}

/// Individual agenda item
#[derive(serde::Deserialize, ToolParameters)]
struct PuntoOrdenDia<'a> {
    #[description("Título del punto, si hay número se quitará")]
    titulo: &'a str,

    #[description("Descripción de lo tratado y decisiones tomadas")]
    contenido: &'a str,
}

/// Post assembly minutes tool
pub struct PostAssemblyMinutes;

impl Tool for PostAssemblyMinutes {
    fn name(&self) -> &'static str {
        "post_assembly_minutes"
    }

    fn description(&self) -> &'static str {
        "Genera y publica el acta de una asamblea en formato PDF en el chat de actas configurado"
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
        let args: CallArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow::anyhow!("Error al procesar los argumentos: {}", e))?;

        // Parse the date
        let assembly_date = match chrono::DateTime::parse_from_rfc3339(args.fecha_hora) {
            Ok(d) => d.with_timezone(&chrono::Utc),
            Err(_) => {
                // Try alternative format without timezone
                match chrono::NaiveDateTime::parse_from_str(args.fecha_hora, "%Y-%m-%dT%H:%M:%S") {
                    Ok(naive) => chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc),
                    Err(e) => {
                        return Ok(ToolCallAction::Message(format!(
                            "❌ Error: Formato de fecha inválido. Use formato ISO 8601 (ejemplo: 2024-03-15T18:00:00). Error: {}",
                            e
                        )))
                    }
                }
            }
        };

        // Get the assembly minutes chat ID
        let minutes_chat_id = match ctx.get_assembly_minutes_chat_id().await {
            Ok(Some(id)) => id,
            Ok(None) => {
                return Ok(ToolCallAction::Message(
                    "❌ No hay ningún chat de actas configurado".to_string(),
                ))
            }
            Err(e) => {
                return Ok(ToolCallAction::Message(format!(
                    "❌ Error al obtener el chat de actas: {}",
                    e
                )))
            }
        };

        // Generate PDF
        let pdf_path = match generate_assembly_pdf(ctx, assembly_date, &args).await {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolCallAction::Message(format!(
                    "❌ Error al generar el PDF del acta: {}",
                    e
                )))
            }
        };

        // Send PDF to the minutes chat

        match ctx
            .bot
            .send_document(ChatId(minutes_chat_id), &pdf_path)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to post assembly minutes: {}", e);
                return Ok(ToolCallAction::Message(format!(
                    "❌ Error al publicar el acta: {}",
                    e
                )));
            }
        }

        // Clean up the temporary PDF file
        let _ = tokio::fs::remove_file(&pdf_path).await;

        // Also send a summary message to the current chat
        let summary = format!(
            "✅ Acta de asamblea del {} publicada correctamente en el chat de actas",
            assembly_date.format("%d/%m/%Y")
        );

        Ok(ToolCallAction::Message(summary))
    }
}

/// Generates a PDF with the assembly minutes
async fn generate_assembly_pdf(
    _ctx: &AssociationContext,
    assembly_date: chrono::DateTime<chrono::Utc>,
    args: &CallArgs<'_>,
) -> Result<PathBuf> {
    pub fn mm_to_em_thousandths(mm: impl Into<f32>, font_size_pt: impl Into<f32>) -> f32 {
        // 1 pt = 1/72 in ; 1 in = 25.4 mm  →  1 mm = 72/25.4 pt
        const PT_PER_MM: f32 = 72.0_f32 / 25.4_f32; // ≈ 2.8346457

        let mm_f: f32 = mm.into();
        let font_f: f32 = font_size_pt.into();

        // Step 1 – mm → pt (PrintPDF’s internal unit)
        let pt_len = mm_f * PT_PER_MM;

        // Step 2 – pt → em (divide by the current font size)
        let em = pt_len / font_f;

        // Step 3 – em → thousandths of an em
        // We keep the result as a float; you can round later if you need an integer.
        em * 1_000.0_f32
    }

    const STANDARD_FONT_SIZE: f32 = 11.0;
    const TITLE2_FONT_SIZE: f32 = 14.0;
    const LEFT_MARGIN: f32 = 20.0;

    use printpdf::{ops::Op, PdfDocument};

    let filename = format!("Acta_Asamblea_{}.pdf", assembly_date.format("%d-%m-%Y"));
    let pdf_path = std::env::temp_dir().join(&filename);

    // Create PDF document
    let mut doc = PdfDocument::new("Acta de Asamblea");

    // Add fonts
    let font_id = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::Helvetica);
    let font_bold_id = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::HelveticaBold);
    let font_italic_id = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::HelveticaOblique);

    let title2_fnt = Op::SetFont {
        font: font_bold_id.clone(),
        size: Pt(TITLE2_FONT_SIZE),
    };

    let normal_text = Op::SetFont {
        font: font_id,
        size: Pt(STANDARD_FONT_SIZE),
    };

    let paragraph = |text: String| Op::ShowText {
        items: vec![
            TextItem::Offset(mm_to_em_thousandths(LEFT_MARGIN, STANDARD_FONT_SIZE)),
            TextItem::Text(text),
        ],
    };

    let title2 = |text: String| Op::ShowText {
        items: vec![
            TextItem::Offset(mm_to_em_thousandths(LEFT_MARGIN, TITLE2_FONT_SIZE)),
            TextItem::Text(text),
        ],
    };

    let mut page_contents = vec![
        // Title
        Op::SetFont {
            font: font_bold_id.clone(),
            size: Pt(16.0),
        },
        Op::ShowText {
            items: vec![
                TextItem::Offset(mm_to_em_thousandths(75.0, 16.0)),
                TextItem::Text("ACTA DE ASAMBLEA".into()),
            ],
        },
        // Header info
        normal_text.clone(),
        paragraph(format!("Fecha: {}", assembly_date.format("%d/%m/%Y"))),
        paragraph(format!("Hora de inicio: {}", assembly_date.format("%H:%M"))),
        paragraph(format!("Canal: {}", args.canal)),
        paragraph(format!("Hora de finalización: {}", args.hora_fin)),
        // Index
        title2_fnt.clone(),
        title2("ÍNDICE".to_string()),
        normal_text.clone(),
        paragraph("1. Aprobación del acta anterior".to_string()),
        paragraph("2. Informe del presidente".to_string()),
    ];

    // Add agenda items to index
    for (i, punto) in args.puntos_del_orden_dia.iter().enumerate() {
        page_contents.push(paragraph(format!("{}. {}", i + 3, punto.titulo)));
    }
    page_contents.push(paragraph(format!(
        "{}. Ruegos y preguntas",
        args.puntos_del_orden_dia.len() + 3
    )));

    // Point 1: Approval of previous minutes
    page_contents.push(title2_fnt.clone());
    page_contents.push(title2("1. APROBACIÓN DEL ACTA ANTERIOR".to_string()));
    page_contents.push(normal_text.clone());
    page_contents.push(paragraph(format!(
        "A favor: {}",
        args.punto1_aprobacion_acta.a_favor
    )));
    page_contents.push(paragraph(format!(
        "En contra: {}",
        args.punto1_aprobacion_acta.en_contra
    )));
    page_contents.push(paragraph(format!(
        "Abstenciones: {}",
        args.punto1_aprobacion_acta.abstenciones
    )));

    // Point 2: President's report
    page_contents.push(title2_fnt.clone());
    page_contents.push(title2("2. INFORME DEL PRESIDENTE".to_string()));
    page_contents.push(normal_text.clone());
    for line in args.punto2_informe_presidente.lines() {
        page_contents.push(paragraph(line.to_string()));
    }

    // Additional points
    for (i, punto) in args.puntos_del_orden_dia.iter().enumerate() {
        page_contents.push(title2_fnt.clone());
        page_contents.push(title2(format!("{}. {}", i + 3, punto.titulo)));
        page_contents.push(normal_text.clone());
        for line in punto.contenido.lines() {
            page_contents.push(paragraph(line.to_string()));
        }
    }

    // Final point: Questions and requests
    page_contents.push(title2_fnt.clone());
    page_contents.push(title2(format!(
        "{}. RUEGOS Y PREGUNTAS",
        args.puntos_del_orden_dia.len() + 3
    )));
    page_contents.push(normal_text.clone());
    for line in args.ruegos_y_preguntas.lines() {
        page_contents.push(paragraph(line.to_string()));
    }

    // Closing
    page_contents.push(normal_text.clone());
    page_contents.push(Op::SetFont {
        font: font_italic_id,
        size: Pt(STANDARD_FONT_SIZE),
    });
    page_contents.push(paragraph(format!(
        "Se levanta la sesión a las {}",
        args.hora_fin
    )));

    // Signatures
    page_contents.push(normal_text.clone());
    page_contents.push(paragraph(
        "_______________________                                  _______________________"
            .to_string(),
    ));
    page_contents.push(Op::SetFont {
        font: font_bold_id,
        size: Pt(STANDARD_FONT_SIZE),
    });
    page_contents.push(paragraph(
        "       PRESIDENTE                                                    SECRETARIO"
            .to_string(),
    ));

    // Create a page and add content
    let page = printpdf::PdfPage::new(Mm(210.0), Mm(297.0), page_contents);
    let bytes = doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    // Save PDF
    tokio::fs::write(&pdf_path, bytes).await?;

    Ok(pdf_path)
}
