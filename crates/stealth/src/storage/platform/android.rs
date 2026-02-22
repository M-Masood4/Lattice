//! Android Keystore storage implementation for stealth keys
//!
//! This module provides secure storage for stealth key pairs using the Android Keystore.
//! The Android Keystore provides hardware-backed encryption on supported devices and
//! secure storage that persists across app restarts.
//!
//! # Security Features
//! - Hardware-backed encryption via Android Keystore (on supported devices)
//! - Automatic encryption at rest (AES-256-GCM)
//! - Biometric authentication support (fingerprint/face unlock)
//! - TEE (Trusted Execution Environment) protection on supported devices
//! - Automatic key separation (spending and viewing keys stored separately)
//!
//! # Requirements
//! Validates: Requirements 9.1, 9.2, 9.3, 9.6

#[cfg(target_os = "android")]
use crate::error::{StealthError, StealthResult};
#[cfg(target_os = "android")]
use crate::keypair::StealthKeyPair;
#[cfg(target_os = "android")]
use crate::storage::SecureStorage;
#[cfg(target_os = "android")]
use async_trait::async_trait;
#[cfg(target_os = "android")]
use jni::objects::{JClass, JObject, JString, JValue};
#[cfg(target_os = "android")]
use jni::sys::{jbyteArray, jstring};
#[cfg(target_os = "android")]
use jni::{JNIEnv, JavaVM};
#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
/// Android Keystore storage adapter for stealth key pairs
///
/// This implementation uses JNI to interact with the Android Keystore API.
/// Each key pair is stored as separate keystore entries (spending and viewing keys)
/// to enable view-only wallet modes.
///
/// # Android Keystore Structure
/// - Alias: "{keystore_prefix}.{id}.spending" or "{keystore_prefix}.{id}.viewing"
/// - Algorithm: AES/GCM/NoPadding (256-bit)
/// - Purpose: ENCRYPT | DECRYPT
///
/// # Requirements
/// Validates: Requirements 9.1 (platform-specific storage), 9.2 (encryption at rest),
///            9.3 (key separation), 9.6 (no plaintext logging)
pub struct AndroidKeystoreStorage {
    /// JNI Java VM reference
    jvm: Arc<JavaVM>,
    /// Keystore alias prefix
    keystore_prefix: String,
}

#[cfg(target_os = "android")]
impl AndroidKeystoreStorage {
    /// Create a new Android Keystore storage adapter
    ///
    /// # Arguments
    /// * `jvm` - Java VM reference for JNI calls
    /// * `keystore_prefix` - Prefix for keystore aliases (e.g., "com.myapp.stealth")
    ///
    /// # Example
    /// ```no_run
    /// use stealth::storage::android::AndroidKeystoreStorage;
    /// use jni::JavaVM;
    /// use std::sync::Arc;
    ///
    /// # fn example(jvm: Arc<JavaVM>) {
    /// let storage = AndroidKeystoreStorage::new(jvm, "com.myapp.stealth");
    /// # }
    /// ```
    pub fn new(jvm: Arc<JavaVM>, keystore_prefix: &str) -> Self {
        Self {
            jvm,
            keystore_prefix: keystore_prefix.to_string(),
        }
    }

    /// Store data in the Android Keystore using JNI
    ///
    /// This method calls the Android Keystore API via JNI to:
    /// 1. Generate or retrieve an AES key in the Keystore
    /// 2. Encrypt the data using AES-GCM
    /// 3. Store the encrypted data in SharedPreferences
    ///
    /// # Arguments
    /// * `alias` - Keystore alias (unique identifier)
    /// * `data` - Data to encrypt and store
    ///
    /// # Security
    /// The Android Keystore automatically uses hardware-backed encryption on supported devices.
    fn store_keystore_item(&self, alias: &str, data: &[u8]) -> StealthResult<()> {
        let env = self.jvm.attach_current_thread()
            .map_err(|e| StealthError::StorageFailed(format!("Failed to attach JNI thread: {}", e)))?;

        // Call Java helper method: KeystoreHelper.storeData(alias, data)
        let helper_class = env.find_class("com/stealth/KeystoreHelper")
            .map_err(|e| StealthError::StorageFailed(format!("Failed to find KeystoreHelper class: {}", e)))?;

        let alias_jstring = env.new_string(alias)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to create alias string: {}", e)))?;

        let data_jbytearray = env.byte_array_from_slice(data)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to create byte array: {}", e)))?;

        let result = env.call_static_method(
            helper_class,
            "storeData",
            "(Ljava/lang/String;[B)V",
            &[
                JValue::Object(alias_jstring.into()),
                JValue::Object(JObject::from(data_jbytearray)),
            ],
        );

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(StealthError::StorageFailed(format!("Failed to store keystore item: {}", e))),
        }
    }

    /// Load data from the Android Keystore using JNI
    ///
    /// This method calls the Android Keystore API via JNI to:
    /// 1. Retrieve the AES key from the Keystore
    /// 2. Load the encrypted data from SharedPreferences
    /// 3. Decrypt the data using AES-GCM
    ///
    /// # Arguments
    /// * `alias` - Keystore alias (unique identifier)
    ///
    /// # Returns
    /// The decrypted data
    fn load_keystore_item(&self, alias: &str) -> StealthResult<Vec<u8>> {
        let env = self.jvm.attach_current_thread()
            .map_err(|e| StealthError::StorageFailed(format!("Failed to attach JNI thread: {}", e)))?;

        // Call Java helper method: KeystoreHelper.loadData(alias)
        let helper_class = env.find_class("com/stealth/KeystoreHelper")
            .map_err(|e| StealthError::StorageFailed(format!("Failed to find KeystoreHelper class: {}", e)))?;

        let alias_jstring = env.new_string(alias)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to create alias string: {}", e)))?;

        let result = env.call_static_method(
            helper_class,
            "loadData",
            "(Ljava/lang/String;)[B",
            &[JValue::Object(alias_jstring.into())],
        );

        match result {
            Ok(JValue::Object(obj)) => {
                if obj.is_null() {
                    return Err(StealthError::StorageFailed(format!("Keystore item not found: {}", alias)));
                }

                let jbytearray = jbyteArray::from(obj.into_inner());
                let data = env.convert_byte_array(jbytearray)
                    .map_err(|e| StealthError::StorageFailed(format!("Failed to convert byte array: {}", e)))?;

                Ok(data)
            }
            Ok(_) => Err(StealthError::StorageFailed("Unexpected return type from loadData".to_string())),
            Err(e) => Err(StealthError::StorageFailed(format!("Failed to load keystore item: {}", e))),
        }
    }

    /// Delete data from the Android Keystore using JNI
    ///
    /// # Arguments
    /// * `alias` - Keystore alias (unique identifier)
    fn delete_keystore_item(&self, alias: &str) -> StealthResult<()> {
        let env = self.jvm.attach_current_thread()
            .map_err(|e| StealthError::StorageFailed(format!("Failed to attach JNI thread: {}", e)))?;

        // Call Java helper method: KeystoreHelper.deleteData(alias)
        let helper_class = env.find_class("com/stealth/KeystoreHelper")
            .map_err(|e| StealthError::StorageFailed(format!("Failed to find KeystoreHelper class: {}", e)))?;

        let alias_jstring = env.new_string(alias)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to create alias string: {}", e)))?;

        let result = env.call_static_method(
            helper_class,
            "deleteData",
            "(Ljava/lang/String;)V",
            &[JValue::Object(alias_jstring.into())],
        );

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(StealthError::StorageFailed(format!("Failed to delete keystore item: {}", e))),
        }
    }

    /// List all keystore items with the configured prefix
    fn list_keystore_items(&self) -> StealthResult<Vec<String>> {
        let env = self.jvm.attach_current_thread()
            .map_err(|e| StealthError::StorageFailed(format!("Failed to attach JNI thread: {}", e)))?;

        // Call Java helper method: KeystoreHelper.listAliases(prefix)
        let helper_class = env.find_class("com/stealth/KeystoreHelper")
            .map_err(|e| StealthError::StorageFailed(format!("Failed to find KeystoreHelper class: {}", e)))?;

        let prefix_jstring = env.new_string(&self.keystore_prefix)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to create prefix string: {}", e)))?;

        let result = env.call_static_method(
            helper_class,
            "listAliases",
            "(Ljava/lang/String;)[Ljava/lang/String;",
            &[JValue::Object(prefix_jstring.into())],
        );

        match result {
            Ok(JValue::Object(obj)) => {
                if obj.is_null() {
                    return Ok(Vec::new());
                }

                // Convert Java String array to Rust Vec<String>
                let array = obj.into_inner();
                let length = env.get_array_length(array)
                    .map_err(|e| StealthError::StorageFailed(format!("Failed to get array length: {}", e)))?;

                let mut aliases = Vec::new();
                for i in 0..length {
                    let element = env.get_object_array_element(array, i)
                        .map_err(|e| StealthError::StorageFailed(format!("Failed to get array element: {}", e)))?;

                    let jstring = JString::from(element);
                    let rust_string = env.get_string(jstring)
                        .map_err(|e| StealthError::StorageFailed(format!("Failed to convert string: {}", e)))?;

                    aliases.push(rust_string.to_string_lossy().to_string());
                }

                Ok(aliases)
            }
            Ok(_) => Err(StealthError::StorageFailed("Unexpected return type from listAliases".to_string())),
            Err(e) => Err(StealthError::StorageFailed(format!("Failed to list keystore items: {}", e))),
        }
    }
}

#[cfg(target_os = "android")]
#[async_trait]
impl SecureStorage for AndroidKeystoreStorage {
    /// Store a stealth key pair securely in the Android Keystore
    ///
    /// This implementation:
    /// 1. Serializes the keypair components
    /// 2. Stores spending and viewing keys as separate keystore entries
    /// 3. Uses hardware-backed encryption provided by Android Keystore
    ///
    /// # Requirements
    /// Validates: Requirements 9.1 (Android Keystore), 9.2 (encryption at rest),
    ///            9.3 (key separation)
    async fn store_keypair(&self, id: &str, keypair: &StealthKeyPair) -> StealthResult<()> {
        use serde_json;

        // Extract secret keys
        let spending_secret = keypair.spending_keypair().secret.to_bytes();
        let viewing_secret = keypair.viewing_keypair().secret.to_bytes();

        // Serialize keys
        let spending_data = serde_json::to_vec(&spending_secret)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to serialize spending key: {}", e)))?;

        let viewing_data = serde_json::to_vec(&viewing_secret)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to serialize viewing key: {}", e)))?;

        // Store public keys and metadata
        let metadata = serde_json::json!({
            "spending_public": keypair.spending_keypair().public.to_bytes(),
            "viewing_public": keypair.viewing_keypair().public.to_bytes(),
            "version": 1u8,
        });
        let metadata_data = serde_json::to_vec(&metadata)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to serialize metadata: {}", e)))?;

        // Store spending key separately (Requirement 9.3)
        let spending_alias = format!("{}.{}.spending", self.keystore_prefix, id);
        self.store_keystore_item(&spending_alias, &spending_data)?;

        // Store viewing key separately (Requirement 9.3)
        let viewing_alias = format!("{}.{}.viewing", self.keystore_prefix, id);
        self.store_keystore_item(&viewing_alias, &viewing_data)?;

        // Store metadata
        let metadata_alias = format!("{}.{}.metadata", self.keystore_prefix, id);
        self.store_keystore_item(&metadata_alias, &metadata_data)?;

        Ok(())
    }

    /// Load a stealth key pair from the Android Keystore
    ///
    /// This implementation:
    /// 1. Retrieves spending and viewing keys from separate keystore entries
    /// 2. Decrypts the keys (automatic via Android Keystore)
    /// 3. Reconstructs the StealthKeyPair
    ///
    /// # Requirements
    /// Validates: Requirements 9.1 (Android Keystore), 9.3 (key separation)
    async fn load_keypair(&self, id: &str) -> StealthResult<StealthKeyPair> {
        use serde_json;
        use ed25519_dalek::{Keypair, PublicKey, SecretKey};

        // Load spending key
        let spending_alias = format!("{}.{}.spending", self.keystore_prefix, id);
        let spending_data = self.load_keystore_item(&spending_alias)?;
        let spending_secret_bytes: [u8; 32] = serde_json::from_slice(&spending_data)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to deserialize spending key: {}", e)))?;

        // Load viewing key
        let viewing_alias = format!("{}.{}.viewing", self.keystore_prefix, id);
        let viewing_data = self.load_keystore_item(&viewing_alias)?;
        let viewing_secret_bytes: [u8; 32] = serde_json::from_slice(&viewing_data)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to deserialize viewing key: {}", e)))?;

        // Load metadata
        let metadata_alias = format!("{}.{}.metadata", self.keystore_prefix, id);
        let metadata_data = self.load_keystore_item(&metadata_alias)?;
        let metadata: serde_json::Value = serde_json::from_slice(&metadata_data)
            .map_err(|e| StealthError::StorageFailed(format!("Failed to deserialize metadata: {}", e)))?;

        // Extract public keys
        let spending_public_bytes: [u8; 32] = serde_json::from_value(metadata["spending_public"].clone())
            .map_err(|e| StealthError::StorageFailed(format!("Failed to extract spending public key: {}", e)))?;

        let viewing_public_bytes: [u8; 32] = serde_json::from_value(metadata["viewing_public"].clone())
            .map_err(|e| StealthError::StorageFailed(format!("Failed to extract viewing public key: {}", e)))?;

        let version: u8 = serde_json::from_value(metadata["version"].clone())
            .map_err(|e| StealthError::StorageFailed(format!("Failed to extract version: {}", e)))?;

        // Reconstruct keypairs
        let spending_secret = SecretKey::from_bytes(&spending_secret_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid spending secret: {}", e)))?;

        let spending_public = PublicKey::from_bytes(&spending_public_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid spending public key: {}", e)))?;

        let viewing_secret = SecretKey::from_bytes(&viewing_secret_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid viewing secret: {}", e)))?;

        let viewing_public = PublicKey::from_bytes(&viewing_public_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid viewing public key: {}", e)))?;

        let spending_keypair = Keypair {
            secret: spending_secret,
            public: spending_public,
        };

        let viewing_keypair = Keypair {
            secret: viewing_secret,
            public: viewing_public,
        };

        StealthKeyPair::from_parts(spending_keypair, viewing_keypair, version)
    }

    /// Delete a stealth key pair from the Android Keystore
    async fn delete_keypair(&self, id: &str) -> StealthResult<()> {
        // Delete spending key
        let spending_alias = format!("{}.{}.spending", self.keystore_prefix, id);
        self.delete_keystore_item(&spending_alias)?;

        // Delete viewing key
        let viewing_alias = format!("{}.{}.viewing", self.keystore_prefix, id);
        self.delete_keystore_item(&viewing_alias)?;

        // Delete metadata
        let metadata_alias = format!("{}.{}.metadata", self.keystore_prefix, id);
        self.delete_keystore_item(&metadata_alias)?;

        Ok(())
    }

    /// List all stored key pair IDs
    async fn list_keypairs(&self) -> StealthResult<Vec<String>> {
        let aliases = self.list_keystore_items()?;

        // Extract unique IDs (remove prefix and suffixes)
        let prefix_with_dot = format!("{}.", self.keystore_prefix);
        let mut ids = std::collections::HashSet::new();

        for alias in aliases {
            if let Some(remainder) = alias.strip_prefix(&prefix_with_dot) {
                if let Some(id) = remainder.strip_suffix(".spending")
                    .or_else(|| remainder.strip_suffix(".viewing"))
                    .or_else(|| remainder.strip_suffix(".metadata"))
                {
                    ids.insert(id.to_string());
                }
            }
        }

        Ok(ids.into_iter().collect())
    }

    /// Store arbitrary encrypted data in the Android Keystore
    async fn store_data(&self, key: &str, data: &[u8]) -> StealthResult<()> {
        let alias = format!("{}.data.{}", self.keystore_prefix, key);
        self.store_keystore_item(&alias, data)
    }

    /// Load arbitrary encrypted data from the Android Keystore
    async fn load_data(&self, key: &str) -> StealthResult<Vec<u8>> {
        let alias = format!("{}.data.{}", self.keystore_prefix, key);
        self.load_keystore_item(&alias)
    }
}

// Stub implementation for non-Android platforms
#[cfg(not(target_os = "android"))]
pub struct AndroidKeystoreStorage;

#[cfg(not(target_os = "android"))]
impl AndroidKeystoreStorage {
    pub fn new(_jvm: std::sync::Arc<()>, _keystore_prefix: &str) -> Self {
        Self
    }
}
