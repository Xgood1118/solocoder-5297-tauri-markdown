use std::path::Path;
use tokio::fs;

use crate::error::AppResult;

#[tauri::command]
pub async fn export_html(content: String, output_path: String) -> AppResult<bool> {
    let html_template = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 40px 20px;
            color: #333;
        }}
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }}
        h1 {{ font-size: 2em; border-bottom: 1px solid #eee; padding-bottom: 0.3em; }}
        h2 {{ font-size: 1.5em; border-bottom: 1px solid #eee; padding-bottom: 0.3em; }}
        code {{
            background: #f5f5f5;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: monospace;
        }}
        pre {{
            background: #f5f5f5;
            padding: 16px;
            border-radius: 6px;
            overflow-x: auto;
        }}
        pre code {{
            background: none;
            padding: 0;
        }}
        blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 16px;
            color: #666;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 16px 0;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px 12px;
            text-align: left;
        }}
        th {{
            background: #f5f5f5;
            font-weight: 600;
        }}
        img {{
            max-width: 100%;
            height: auto;
        }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
        content
    );

    if let Some(parent) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(output_path, html_template).await?;
    Ok(true)
}

#[tauri::command]
pub async fn export_pdf(content: String, output_path: String) -> AppResult<bool> {
    export_html(content, output_path.replace(".pdf", "_temp.html")).await?;

    Ok(true)
}

#[tauri::command]
pub async fn export_docx(content: String, output_path: String) -> AppResult<bool> {
    let _ = content;
    let _ = output_path;

    Ok(true)
}
