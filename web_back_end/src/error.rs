pub struct WebBackEndError(eyre::Report);

impl<T> From<T> for WebBackEndError
where
    T: Into<eyre::Report>,
{
    fn from(error: T) -> Self {
        Self(error.into())
    }
}

impl axum::response::IntoResponse for WebBackEndError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Web back end error: {}", self.0),
        )
            .into_response()
    }
}
