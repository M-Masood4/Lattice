// Session Manager - manages discovery sessions and timeouts

use crate::{DiscoveryMethod, DiscoverySession, ProximityError, Result};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration as TokioDuration};
use uuid::Uuid;

/// Default session duration in minutes
const DEFAULT_SESSION_DURATION_MINUTES: i64 = 30;

/// Session extension increment in minutes
const SESSION_EXTENSION_MINUTES: i64 = 15;

/// Cleanup interval in seconds
const CLEANUP_INTERVAL_SECONDS: u64 = 60;

pub struct SessionManager {
    pub(crate) active_sessions: Arc<RwLock<HashMap<Uuid, DiscoverySession>>>,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            cleanup_handle: None,
        }
    }

    /// Start the background cleanup task
    pub fn start_cleanup_task(&mut self) {
        let sessions = Arc::clone(&self.active_sessions);
        
        let handle = tokio::spawn(async move {
            let mut cleanup_interval = interval(TokioDuration::from_secs(CLEANUP_INTERVAL_SECONDS));
            
            loop {
                cleanup_interval.tick().await;
                
                // Perform cleanup
                let mut sessions_lock = sessions.write().await;
                let now = Utc::now();
                
                // Find expired sessions
                let expired_session_ids: Vec<Uuid> = sessions_lock
                    .iter()
                    .filter(|(_, session)| session.expires_at <= now)
                    .map(|(id, _)| *id)
                    .collect();

                // Remove expired sessions
                for session_id in expired_session_ids {
                    sessions_lock.remove(&session_id);
                }
            }
        });
        
        self.cleanup_handle = Some(handle);
    }

    /// Stop the background cleanup task
    pub fn stop_cleanup_task(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
    }

    /// Start a new discovery session with configurable duration
    /// Default duration is 30 minutes if duration_minutes is 0
    pub async fn start_session(
        &self,
        user_id: Uuid,
        method: DiscoveryMethod,
        duration_minutes: u32,
    ) -> Result<DiscoverySession> {
        let session_id = Uuid::new_v4();
        let started_at = Utc::now();
        
        // Use default duration if 0 is provided
        let duration = if duration_minutes == 0 {
            DEFAULT_SESSION_DURATION_MINUTES
        } else {
            duration_minutes as i64
        };
        
        let expires_at = started_at + Duration::minutes(duration);

        let session = DiscoverySession {
            session_id,
            user_id,
            discovery_method: method,
            started_at,
            expires_at,
            auto_extend: false,
        };

        // Store session in HashMap
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id, session.clone());

        Ok(session)
    }

    /// Extend an existing session by 15 minutes
    pub async fn extend_session(&self, session_id: Uuid, additional_minutes: u32) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        
        // Validate session exists
        let session = sessions.get_mut(&session_id)
            .ok_or_else(|| ProximityError::SessionNotFound(session_id))?;

        // Use default extension if 0 is provided
        let extension = if additional_minutes == 0 {
            SESSION_EXTENSION_MINUTES
        } else {
            additional_minutes as i64
        };

        // Update expiration time
        session.expires_at = session.expires_at + Duration::minutes(extension);

        Ok(())
    }

    /// End a discovery session
    pub async fn end_session(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        
        // Remove session from HashMap
        sessions.remove(&session_id)
            .ok_or_else(|| ProximityError::SessionNotFound(session_id))?;

        Ok(())
    }

    /// Clean up expired sessions and return count of removed sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let mut sessions = self.active_sessions.write().await;
        let now = Utc::now();
        
        // Find expired sessions
        let expired_session_ids: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, session)| session.expires_at <= now)
            .map(|(id, _)| *id)
            .collect();

        let count = expired_session_ids.len() as u64;

        // Remove expired sessions
        for session_id in expired_session_ids {
            sessions.remove(&session_id);
        }

        Ok(count)
    }

    /// Get an active session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<DiscoverySession> {
        let sessions = self.active_sessions.read().await;
        sessions.get(&session_id)
            .cloned()
            .ok_or_else(|| ProximityError::SessionNotFound(session_id))
    }

    /// Check if a session is expired
    pub async fn is_session_expired(&self, session_id: Uuid) -> Result<bool> {
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(&session_id)
            .ok_or_else(|| ProximityError::SessionNotFound(session_id))?;
        
        Ok(session.expires_at <= Utc::now())
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<DiscoverySession>> {
        let sessions = self.active_sessions.read().await;
        let user_sessions: Vec<DiscoverySession> = sessions
            .values()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect();
        
        Ok(user_sessions)
    }

    /// Helper method for testing: manually set session expiry
    pub async fn set_session_expiry_for_testing(&self, session_id: Uuid, expires_at: chrono::DateTime<Utc>) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or_else(|| ProximityError::SessionNotFound(session_id))?;
        session.expires_at = expires_at;
        Ok(())
    }
}
