use crate::Cache;
use chrono::Utc;
use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec, TokenReviewStatus};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::RwLock;
use tower::Service;

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
}

impl Service<TokenReviewSpec> for &Client {
    type Response = Option<TokenReviewStatus>;
    type Error = kube::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: TokenReviewSpec) -> Self::Future {
        let client = self.client.clone();
        let cache = self.cache.clone();
        Box::pin(async move {
            if crate::exp(&request) > Some(Utc::now().timestamp()) {
                let status = cache.read().await.get(&request).cloned();
                if let Some(status) = status {
                    Ok(Some(status))
                } else {
                    let status = kube::Api::all(client)
                        .create(
                            &Default::default(),
                            &TokenReview {
                                spec: request.clone(),
                                ..Default::default()
                            },
                        )
                        .await?
                        .status;
                    if let Some(status) = &status {
                        cache.write().await.put(request, status.clone());
                    }
                    Ok(status)
                }
            } else {
                Ok(None)
            }
        })
    }
}

impl Service<TokenReviewSpec> for Client {
    type Response = Option<TokenReviewStatus>;
    type Error = kube::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (&*self).poll_ready(cx)
    }

    fn call(&mut self, request: TokenReviewSpec) -> Self::Future {
        (&*self).call(request)
    }
}
