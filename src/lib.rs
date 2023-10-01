mod cache;
mod client;

use base64::prelude::{Engine, BASE64_STANDARD_NO_PAD};
pub use cache::Cache;
pub use client::Client;
pub use k8s_openapi;
use k8s_openapi::api::authentication::v1::TokenReviewSpec;
pub use kube;
pub use kube::Error;
use serde::Deserialize;

fn exp(spec: &TokenReviewSpec) -> Option<i64> {
    #[derive(Deserialize)]
    struct Payload {
        exp: i64,
    }

    let payload = spec.token.as_ref()?.split('.').nth(1)?;
    Some(
        serde_json::from_slice::<Payload>(&BASE64_STANDARD_NO_PAD.decode(payload).ok()?)
            .ok()?
            .exp,
    )
}
