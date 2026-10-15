use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::time;
use tracing::warn;

use super::pages::post_page::{ApprovedComment, fetch_approved_comments};

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);
const FETCH_TIMEOUT: Duration = Duration::from_secs(55);

static COMMENTS: Lazy<Mutex<HashMap<String, CachedComments>>> = Lazy::new(Default::default);

#[derive(Default)]
struct CachedComments {
    comments: Vec<ApprovedComment>,
    last_refresh: Option<Instant>,
}

impl CachedComments {
    fn start_refresh(&mut self, now: Instant) -> bool {
        if self
            .last_refresh
            .is_some_and(|last| now.duration_since(last) < REFRESH_INTERVAL)
        {
            return false;
        }
        self.last_refresh = Some(now);
        true
    }
}

pub fn cached_approved_comments(post_id: String) -> Vec<ApprovedComment> {
    let (comments, refresh) = {
        let mut cache = COMMENTS.lock();
        let entry = cache.entry(post_id.clone()).or_default();
        (entry.comments.clone(), entry.start_refresh(Instant::now()))
    };

    if refresh {
        tokio::spawn(async move {
            match time::timeout(FETCH_TIMEOUT, fetch_approved_comments(post_id.clone())).await {
                Ok(Ok(comments)) => {
                    if let Some(entry) = COMMENTS.lock().get_mut(&post_id) {
                        entry.comments = comments;
                    }
                }
                Ok(Err(error)) => warn!(%post_id, "failed to refresh comments: {error}"),
                Err(_) => warn!(%post_id, "comment refresh timed out; keeping cached comments"),
            }
        });
    }

    comments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_refreshes_are_not_started_again_by_page_requests() {
        let now = Instant::now();
        let mut cache = CachedComments::default();
        assert!(cache.start_refresh(now));
        assert!(!cache.start_refresh(now + Duration::from_secs(1)));
        assert!(!cache.start_refresh(now + FETCH_TIMEOUT));
        assert!(cache.start_refresh(now + REFRESH_INTERVAL));
    }
}
