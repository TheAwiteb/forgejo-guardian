use reqwest::{Client, Method};
use url::Url;

use crate::{
    error::{GuardError, GuardResult},
    forgejo_api,
};

/// Returns whether the heatmap is empty.
pub async fn is_empty_heatmap(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
) -> GuardResult<bool> {
    let req = forgejo_api::build_request(
        Method::GET,
        instance,
        token,
        &format!("/api/v1/users/{username}/heatmap"),
    );
    let url = req.url().clone();
    let res = client.execute(req).await?;

    if !res.status().is_success() {
        return Err(GuardError::InvalidForgejoResponse(
            format!("Status code: {status}", status = res.status()),
            url,
        ));
    }

    tracing::debug!("Get heatmap response: {res:?}");

    Ok(res.text().await.unwrap_or_default().trim() == "[]")
}
