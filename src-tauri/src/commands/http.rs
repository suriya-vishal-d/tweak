use crate::http::{execute_request, HttpError, HttpRequest, HttpResponse};

/// Tauri command that sends an HTTP request using reqwest with configurable timeout.
#[tauri::command]
pub async fn send_http_request(request: HttpRequest) -> Result<HttpResponse, HttpError> {
    execute_request(request).await
}

