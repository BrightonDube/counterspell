//! Password hashing with Argon2id, the password policy, and a gate that limits how much hashing
//! runs at once. Hashing is CPU heavy, so it runs on blocking threads, never on the async workers.

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, OnceLock,
    },
    time::Duration,
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use super::errors::AuthError;

/// Argon2id memory cost in KiB (19 MiB).
pub const MEMORY_KIB: u32 = 19_456;
/// Argon2id iterations.
pub const TIME_COST: u32 = 2;
/// Argon2id lanes.
pub const PARALLELISM: u32 = 1;
/// The shortest accepted password, in characters.
pub const MIN_PASSWORD_CHARS: usize = 10;
/// The longest accepted password, in characters.
pub const MAX_PASSWORD_CHARS: usize = 128;
/// The longest accepted password, in bytes. Stops very long inputs from costing time.
pub const MAX_PASSWORD_BYTES: usize = 1024;

/// What a password check found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The password matches and the stored hash uses the current parameters.
    Verified,
    /// The password matches, but the stored hash uses older parameters and should be replaced.
    NeedsRehash,
    /// The password does not match, or the stored value is not a valid hash.
    Rejected,
}

fn argon2() -> Result<Argon2<'static>, AuthError> {
    let params =
        Params::new(MEMORY_KIB, TIME_COST, PARALLELISM, None).map_err(|_| AuthError::Internal)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Checks a password against the policy: 10 to 128 characters and at most 1024 bytes.
///
/// # Errors
///
/// Returns [`AuthError::WeakPassword`] when the password is outside the policy.
pub fn check_policy(password: &str) -> Result<(), AuthError> {
    let characters = password.chars().count();
    if (MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&characters)
        && password.len() <= MAX_PASSWORD_BYTES
    {
        Ok(())
    } else {
        Err(AuthError::WeakPassword)
    }
}

/// Hashes a password into a PHC string with a fresh random salt.
///
/// # Errors
///
/// Returns [`AuthError::Internal`] when the random source or the hasher fails.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    argon2()?
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AuthError::Internal)
}

/// Checks a password against a stored PHC string. Says whether the stored hash is out of date.
#[must_use]
pub fn verify_password(password: &str, stored: &str) -> Verdict {
    let Ok(parsed) = PasswordHash::new(stored) else {
        return Verdict::Rejected;
    };
    let Ok(hasher) = argon2() else {
        return Verdict::Rejected;
    };
    if hasher
        .verify_password(password.as_bytes(), &parsed)
        .is_err()
    {
        return Verdict::Rejected;
    }
    if parameters_are_current(&parsed) {
        Verdict::Verified
    } else {
        Verdict::NeedsRehash
    }
}

fn parameters_are_current(parsed: &PasswordHash) -> bool {
    Params::try_from(parsed).is_ok_and(|params| {
        params.m_cost() == MEMORY_KIB
            && params.t_cost() == TIME_COST
            && params.p_cost() == PARALLELISM
    })
}

/// A hash with the current parameters that matches no real password. Checking against it costs
/// the same time as a real check, so an unknown address takes as long as a wrong password.
fn dummy_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| hash_password("not-a-real-password-anywhere").unwrap_or_default())
}

/// Limits how many hash or verify jobs run at once, and how long a request waits for a turn.
#[derive(Clone)]
pub struct HashGate {
    permits: Arc<Semaphore>,
    wait: Duration,
    started: Arc<AtomicUsize>,
}

impl HashGate {
    /// Builds a gate with `permits` jobs at once. A request waits at most `wait` for a turn.
    #[must_use]
    pub fn new(permits: usize, wait: Duration) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(permits)),
            wait,
            started: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// How many verify or hash jobs have started through this gate. Tests use it to prove that a
    /// limited request never reached the hasher.
    #[must_use]
    pub fn verifications_started(&self) -> usize {
        self.started.load(Ordering::SeqCst)
    }

    /// Waits for a turn. Hold the permit while the work runs.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Busy`] when no turn comes within the wait time.
    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, AuthError> {
        let turn = tokio::time::timeout(self.wait, Arc::clone(&self.permits).acquire_owned());
        match turn.await {
            Ok(Ok(permit)) => Ok(permit),
            Ok(Err(_)) => Err(AuthError::Internal),
            Err(_) => Err(AuthError::Busy),
        }
    }

    /// Checks a password against a stored hash, off the async threads and behind the gate.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Busy`] when the gate is full, and [`AuthError::Internal`] when the
    /// blocking task fails.
    pub async fn verify(&self, password: String, stored: String) -> Result<Verdict, AuthError> {
        self.started.fetch_add(1, Ordering::SeqCst);
        let _turn = self.acquire().await?;
        tokio::task::spawn_blocking(move || verify_password(&password, &stored))
            .await
            .map_err(|_| AuthError::Internal)
    }

    /// Does the same work as [`HashGate::verify`] against a dummy hash, for an unknown address.
    /// The answer is always [`Verdict::Rejected`].
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`HashGate::verify`].
    pub async fn verify_dummy(&self, password: String) -> Result<Verdict, AuthError> {
        self.verify(password, dummy_hash().to_owned()).await?;
        Ok(Verdict::Rejected)
    }

    /// Hashes a password off the async threads and behind the gate.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Busy`] when the gate is full, and [`AuthError::Internal`] otherwise.
    pub async fn hash(&self, password: String) -> Result<String, AuthError> {
        let _turn = self.acquire().await?;
        tokio::task::spawn_blocking(move || hash_password(&password))
            .await
            .map_err(|_| AuthError::Internal)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hash_round_trips_and_a_wrong_password_is_rejected() {
        let stored = hash_password("correct horse battery").expect("hash");
        assert_eq!(
            verify_password("correct horse battery", &stored),
            Verdict::Verified
        );
        assert_eq!(
            verify_password("wrong horse battery", &stored),
            Verdict::Rejected
        );
    }

    #[test]
    fn a_hash_uses_argon2id_with_at_least_the_minimum_cost() {
        let stored = hash_password("correct horse battery").expect("hash");
        assert!(stored.starts_with("$argon2id$"), "got {stored}");
        let parsed = PasswordHash::new(&stored).expect("PHC string");
        let params = Params::try_from(&parsed).expect("parameters");
        assert!(params.m_cost() >= MEMORY_KIB);
        assert!(params.t_cost() >= TIME_COST);
        assert!(params.p_cost() >= PARALLELISM);
    }

    #[test]
    fn a_hash_with_older_parameters_asks_for_a_rehash() {
        let old = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(8 * 1024, 1, 1, None).expect("params"),
        );
        let salt = SaltString::generate(&mut OsRng);
        let stored = old
            .hash_password(b"correct horse battery", &salt)
            .expect("hash")
            .to_string();
        assert_eq!(
            verify_password("correct horse battery", &stored),
            Verdict::NeedsRehash
        );
    }

    #[test]
    fn the_policy_counts_characters_and_bytes() {
        assert!(
            check_policy("0123456789").is_ok(),
            "ten characters is the minimum"
        );
        assert_eq!(check_policy("012345678"), Err(AuthError::WeakPassword));
        assert!(check_policy(&"a".repeat(128)).is_ok());
        assert_eq!(check_policy(&"a".repeat(129)), Err(AuthError::WeakPassword));
        // 128 characters of a 3-byte symbol is 384 bytes: inside the byte limit.
        assert!(check_policy(&"€".repeat(128)).is_ok());
    }
}
