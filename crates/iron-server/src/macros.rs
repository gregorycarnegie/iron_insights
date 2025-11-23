//! Utility macros for reducing boilerplate in handlers and error handling.

/// Creates a simple page handler that just calls a render function.
///
/// # Example
/// ```ignore
/// simple_page_handler!(
///     /// Home page - landing page with overview
///     serve_index => render_index
/// );
/// ```
///
/// Expands to:
/// ```ignore
/// /// Home page - landing page with overview
/// #[instrument(skip(_state))]
/// pub async fn serve_index(State(_state): State<AppState>) -> Markup {
///     render_index()
/// }
/// ```
#[macro_export]
macro_rules! simple_page_handler {
    ($(#[$meta:meta])* $fn_name:ident => $render_fn:ident) => {
        $(#[$meta])*
        #[instrument(skip(state))]
        pub async fn $fn_name(State(state): State<AppState>) -> Markup {
            $render_fn(&state.manifest)
        }
    };
}

/// Handles the common DuckDB handler pattern with error handling and JSON response.
///
/// This macro encapsulates the common pattern of:
/// 1. Checking if DuckDB is available
/// 2. Spawning a blocking task to call a DuckDB method
/// 3. Handling join errors and DuckDB errors
/// 4. Returning a JSON response with metadata
///
/// # Example
/// ```ignore
/// duckdb_handler!(state, calculate_dots_percentiles, "percentiles")
/// ```
///
/// Expands to a full handler implementation with proper error handling.
#[macro_export]
macro_rules! duckdb_handler {
    ($state:expr, $method:ident, $result_key:literal) => {{
        let duckdb = $state.duckdb.as_ref().ok_or_else(|| {
            error!("DuckDB not available");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

        let result = tokio::task::spawn_blocking({
            let duckdb = duckdb.clone();
            move || duckdb.$method()
        })
        .await
        .map_err(|e| {
            error!("Task join error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map_err(|e| {
            error!("DuckDB {} error: {}", $result_key, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(Json(serde_json::json!({
            $result_key: result,
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "engine": "duckdb"
        })))
    }};
}

/// Variant of duckdb_handler that accepts parameters for the DuckDB method.
///
/// # Example
/// ```ignore
/// duckdb_handler_with_params!(state, get_weight_distribution, "distribution", params)
/// ```
#[macro_export]
macro_rules! duckdb_handler_with_params {
    ($state:expr, $method:ident, $result_key:literal, $params:expr) => {{
        let duckdb = $state.duckdb.as_ref().ok_or_else(|| {
            error!("DuckDB not available");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

        let result = tokio::task::spawn_blocking({
            let duckdb = duckdb.clone();
            let params = $params.clone();
            move || duckdb.$method(&params)
        })
        .await
        .map_err(|e| {
            error!("Task join error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map_err(|e| {
            error!("DuckDB {} error: {}", $result_key, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(Json(serde_json::json!({
            $result_key: result,
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "engine": "duckdb"
        })))
    }};
}

/// Logs an error and maps it to HTTP 500 Internal Server Error.
///
/// # Example
/// ```ignore
/// log_and_500!(compute_viz(&state.data, &params, &config), "Compute error")?;
/// ```
///
/// Expands to:
/// ```ignore
/// compute_viz(&state.data, &params, &config).map_err(|e| {
///     error!("Compute error: {}", e);
///     StatusCode::INTERNAL_SERVER_ERROR
/// })?;
/// ```
#[macro_export]
macro_rules! log_and_500 {
    ($result:expr, $context:literal) => {
        $result.map_err(|e| {
            error!(concat!($context, ": {}"), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    };
}

/// Variant that returns the result without the `?` operator.
///
/// Useful when you need to handle the error differently or chain more operations.
#[macro_export]
macro_rules! log_and_500_result {
    ($result:expr, $context:literal) => {
        $result.map_err(|e| {
            error!(concat!($context, ": {}"), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    };
}
