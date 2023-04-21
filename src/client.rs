use crate::Cache;
use chrono::Utc;
use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec, TokenReviewStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Client {
    client: kube::Client,
    cache: Arc<RwLock<Cache>>,
}

impl Client {
    pub fn new(client: kube::Client) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(Cache::new())),
        }
    }

    pub async fn try_default() -> Result<Self, kube::Error> {
        Ok(Self::new(kube::Client::try_default().await?))
    }

    pub async fn call(
        &self,
        spec: TokenReviewSpec,
    ) -> Result<Option<TokenReviewStatus>, kube::Error> {
        if crate::exp(&spec) > Some(Utc::now().timestamp()) {
            let status = self.cache.read().await.get(&spec).cloned();
            if let Some(status) = status {
                Ok(Some(status))
            } else {
                let status = kube::Api::all(self.client.clone())
                    .create(
                        &Default::default(),
                        &TokenReview {
                            spec: spec.clone(),
                            ..Default::default()
                        },
                    )
                    .await?
                    .status
                    .unwrap();
                self.cache.write().await.put(spec, status.clone());
                Ok(Some(status))
            }
        } else {
            Ok(None)
        }
    }
}
