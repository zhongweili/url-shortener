use axum_macros::debug_handler;

#[debug_handler]
pub async fn shorten_handler() -> &'static str {
    "shorten"
}

pub async fn redirect_handler() -> &'static str {
    "redirect"
}
