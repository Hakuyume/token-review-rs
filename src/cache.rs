use chrono::Utc;
use k8s_openapi::api::authentication::v1::{TokenReviewSpec, TokenReviewStatus};
use ref_cast::RefCast;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct Cache {
    map: HashMap<Spec, TokenReviewStatus>,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, spec: &TokenReviewSpec) -> Option<&TokenReviewStatus> {
        if crate::exp(spec) > Some(Utc::now().timestamp()) {
            self.map.get(Spec::ref_cast(spec))
        } else {
            None
        }
    }

    pub fn put(&mut self, spec: TokenReviewSpec, status: TokenReviewStatus) {
        self.map.insert(Spec(spec), status);

        let now = Utc::now().timestamp();
        self.map.retain(|spec, _| crate::exp(&spec.0) > Some(now));
    }
}

#[derive(RefCast)]
#[repr(transparent)]
#[derive(Clone, PartialEq)]
struct Spec(TokenReviewSpec);

impl Eq for Spec {}

impl Hash for Spec {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let TokenReviewSpec { audiences, token } = &self.0;
        audiences.hash(state);
        token.hash(state);
    }
}
