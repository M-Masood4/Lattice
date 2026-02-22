//! Store-and-forward queue for offline recipients

use crate::error::{MeshError, MeshResult};
use crate::router::{DeviceId, MeshPacket};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Store-and-forward queue for mesh packets
///
/// This queue stores packets destined for offline recipients and delivers them
/// when the recipient comes online. Packets are stored per recipient with
/// configurable size limits and expiration times.
pub struct StoreForwardQueue {
    queue: HashMap<DeviceId, VecDeque<MeshPacket>>,
    max_queue_size: usize,
    max_packet_age: Duration,
}

impl StoreForwardQueue {
    /// Create a new store-and-forward queue
    ///
    /// # Arguments
    /// * `max_queue_size` - Maximum number of packets to store per recipient
    /// * `max_packet_age` - Maximum age of packets before they expire
    pub fn new(max_queue_size: usize, max_packet_age: Duration) -> Self {
        info!(
            "Initializing StoreForwardQueue with max_queue_size={}, max_packet_age={:?}",
            max_queue_size, max_packet_age
        );
        
        Self {
            queue: HashMap::new(),
            max_queue_size,
            max_packet_age,
        }
    }

    /// Store packet for offline recipient
    ///
    /// Adds a packet to the queue for the specified recipient. If the queue
    /// for this recipient is full, returns an error.
    ///
    /// # Arguments
    /// * `recipient` - The device ID of the offline recipient
    /// * `packet` - The mesh packet to store
    ///
    /// # Returns
    /// * `Ok(())` if the packet was stored successfully
    /// * `Err(MeshError::QueueFull)` if the recipient's queue is at capacity
    pub fn store(&mut self, recipient: DeviceId, packet: MeshPacket) -> MeshResult<()> {
        debug!(
            "Storing packet {} for offline recipient {}",
            packet.id, recipient
        );
        
        // Get or create queue for this recipient
        let queue = self.queue.entry(recipient).or_insert_with(VecDeque::new);
        
        // Check if queue is full
        if queue.len() >= self.max_queue_size {
            warn!(
                "Queue full for recipient {} (size: {})",
                recipient,
                queue.len()
            );
            return Err(MeshError::QueueFull);
        }
        
        // Add packet to queue
        queue.push_back(packet.clone());
        
        info!(
            "Packet {} stored for recipient {} (queue size: {})",
            packet.id,
            recipient,
            queue.len()
        );
        
        Ok(())
    }

    /// Retrieve packets for recipient when they come online
    ///
    /// Returns all stored packets for the specified recipient and removes them
    /// from the queue. If no packets are stored for this recipient, returns
    /// an empty vector.
    ///
    /// # Arguments
    /// * `recipient` - The device ID of the recipient that came online
    ///
    /// # Returns
    /// A vector of all packets stored for this recipient
    pub fn retrieve(&mut self, recipient: &DeviceId) -> Vec<MeshPacket> {
        if let Some(queue) = self.queue.remove(recipient) {
            let packet_count = queue.len();
            let packets: Vec<MeshPacket> = queue.into_iter().collect();
            
            info!(
                "Retrieved {} packets for recipient {}",
                packet_count, recipient
            );
            
            packets
        } else {
            debug!("No packets stored for recipient {}", recipient);
            Vec::new()
        }
    }

    /// Clean expired packets
    ///
    /// Removes packets that have exceeded the maximum age from all queues.
    /// Also removes empty queues after cleanup.
    pub fn cleanup_expired(&mut self) {
        let now = SystemTime::now();
        let max_age = self.max_packet_age;
        let mut total_removed = 0;
        let mut recipients_to_remove = Vec::new();
        
        debug!("Starting cleanup of expired packets");
        
        // Iterate through all recipient queues
        for (recipient, queue) in self.queue.iter_mut() {
            let original_size = queue.len();
            
            // Remove expired packets
            queue.retain(|packet| {
                if let Ok(age) = now.duration_since(packet.timestamp) {
                    if age > max_age {
                        debug!(
                            "Removing expired packet {} (age: {:?})",
                            packet.id, age
                        );
                        false
                    } else {
                        true
                    }
                } else {
                    // Packet timestamp is in the future (clock skew), keep it
                    warn!(
                        "Packet {} has future timestamp, keeping it",
                        packet.id
                    );
                    true
                }
            });
            
            let removed = original_size - queue.len();
            total_removed += removed;
            
            if removed > 0 {
                info!(
                    "Removed {} expired packets for recipient {}",
                    removed, recipient
                );
            }
            
            // Mark empty queues for removal
            if queue.is_empty() {
                recipients_to_remove.push(*recipient);
            }
        }
        
        // Remove empty queues
        for recipient in recipients_to_remove {
            self.queue.remove(&recipient);
            debug!("Removed empty queue for recipient {}", recipient);
        }
        
        if total_removed > 0 {
            info!("Cleanup complete: removed {} expired packets", total_removed);
        } else {
            debug!("Cleanup complete: no expired packets found");
        }
    }
    
    /// Get the number of packets stored for a specific recipient
    ///
    /// # Arguments
    /// * `recipient` - The device ID of the recipient
    ///
    /// # Returns
    /// The number of packets stored for this recipient
    pub fn get_queue_size(&self, recipient: &DeviceId) -> usize {
        self.queue.get(recipient).map(|q| q.len()).unwrap_or(0)
    }
    
    /// Get the total number of packets stored across all recipients
    ///
    /// # Returns
    /// The total number of packets in the store-and-forward queue
    pub fn total_packets(&self) -> usize {
        self.queue.values().map(|q| q.len()).sum()
    }
    
    /// Get the number of recipients with stored packets
    ///
    /// # Returns
    /// The number of recipients that have packets in the queue
    pub fn recipient_count(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use uuid::Uuid;

    fn create_test_packet(source: DeviceId, destination: Option<DeviceId>) -> MeshPacket {
        MeshPacket {
            id: Uuid::new_v4(),
            source,
            destination,
            ttl: 5,
            payload: vec![1, 2, 3, 4, 5],
            timestamp: SystemTime::now(),
        }
    }

    #[test]
    fn test_store_and_retrieve_single_packet() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();
        let packet = create_test_packet(source, Some(recipient));
        let packet_id = packet.id;

        // Store packet
        assert!(queue.store(recipient, packet).is_ok());
        assert_eq!(queue.get_queue_size(&recipient), 1);

        // Retrieve packet
        let packets = queue.retrieve(&recipient);
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].id, packet_id);

        // Queue should be empty after retrieval
        assert_eq!(queue.get_queue_size(&recipient), 0);
    }

    #[test]
    fn test_store_multiple_packets_same_recipient() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Store 3 packets
        for _ in 0..3 {
            let packet = create_test_packet(source, Some(recipient));
            assert!(queue.store(recipient, packet).is_ok());
        }

        assert_eq!(queue.get_queue_size(&recipient), 3);

        // Retrieve all packets
        let packets = queue.retrieve(&recipient);
        assert_eq!(packets.len(), 3);
        assert_eq!(queue.get_queue_size(&recipient), 0);
    }

    #[test]
    fn test_store_multiple_recipients() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient1 = Uuid::new_v4();
        let recipient2 = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Store packets for recipient1
        for _ in 0..2 {
            let packet = create_test_packet(source, Some(recipient1));
            assert!(queue.store(recipient1, packet).is_ok());
        }

        // Store packets for recipient2
        for _ in 0..3 {
            let packet = create_test_packet(source, Some(recipient2));
            assert!(queue.store(recipient2, packet).is_ok());
        }

        assert_eq!(queue.get_queue_size(&recipient1), 2);
        assert_eq!(queue.get_queue_size(&recipient2), 3);
        assert_eq!(queue.total_packets(), 5);
        assert_eq!(queue.recipient_count(), 2);

        // Retrieve packets for recipient1
        let packets1 = queue.retrieve(&recipient1);
        assert_eq!(packets1.len(), 2);
        assert_eq!(queue.get_queue_size(&recipient1), 0);

        // Recipient2 packets should still be there
        assert_eq!(queue.get_queue_size(&recipient2), 3);
    }

    #[test]
    fn test_queue_full_error() {
        let mut queue = StoreForwardQueue::new(3, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Store 3 packets (max capacity)
        for _ in 0..3 {
            let packet = create_test_packet(source, Some(recipient));
            assert!(queue.store(recipient, packet).is_ok());
        }

        // 4th packet should fail
        let packet = create_test_packet(source, Some(recipient));
        let result = queue.store(recipient, packet);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MeshError::QueueFull));
    }

    #[test]
    fn test_retrieve_nonexistent_recipient() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();

        // Retrieve from empty queue
        let packets = queue.retrieve(&recipient);
        assert_eq!(packets.len(), 0);
    }

    #[test]
    fn test_cleanup_expired_packets() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(1));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Create packet with old timestamp
        let old_packet = MeshPacket {
            id: Uuid::new_v4(),
            source,
            destination: Some(recipient),
            ttl: 5,
            payload: vec![1, 2, 3],
            timestamp: SystemTime::now() - Duration::from_secs(5),
        };

        // Create packet with recent timestamp
        let recent_packet = create_test_packet(source, Some(recipient));

        // Store both packets
        assert!(queue.store(recipient, old_packet).is_ok());
        assert!(queue.store(recipient, recent_packet).is_ok());
        assert_eq!(queue.get_queue_size(&recipient), 2);

        // Cleanup should remove old packet
        queue.cleanup_expired();
        assert_eq!(queue.get_queue_size(&recipient), 1);
    }

    #[test]
    fn test_cleanup_removes_empty_queues() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(1));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Create packet with old timestamp
        let old_packet = MeshPacket {
            id: Uuid::new_v4(),
            source,
            destination: Some(recipient),
            ttl: 5,
            payload: vec![1, 2, 3],
            timestamp: SystemTime::now() - Duration::from_secs(5),
        };

        // Store old packet
        assert!(queue.store(recipient, old_packet).is_ok());
        assert_eq!(queue.recipient_count(), 1);

        // Cleanup should remove packet and empty queue
        queue.cleanup_expired();
        assert_eq!(queue.recipient_count(), 0);
        assert_eq!(queue.total_packets(), 0);
    }

    #[test]
    fn test_cleanup_no_expired_packets() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Store recent packets
        for _ in 0..3 {
            let packet = create_test_packet(source, Some(recipient));
            assert!(queue.store(recipient, packet).is_ok());
        }

        let initial_count = queue.get_queue_size(&recipient);

        // Cleanup should not remove any packets
        queue.cleanup_expired();
        assert_eq!(queue.get_queue_size(&recipient), initial_count);
    }

    #[test]
    fn test_total_packets_and_recipient_count() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient1 = Uuid::new_v4();
        let recipient2 = Uuid::new_v4();
        let recipient3 = Uuid::new_v4();
        let source = Uuid::new_v4();

        assert_eq!(queue.total_packets(), 0);
        assert_eq!(queue.recipient_count(), 0);

        // Add packets for recipient1
        for _ in 0..2 {
            let packet = create_test_packet(source, Some(recipient1));
            queue.store(recipient1, packet).unwrap();
        }

        // Add packets for recipient2
        for _ in 0..3 {
            let packet = create_test_packet(source, Some(recipient2));
            queue.store(recipient2, packet).unwrap();
        }

        // Add packets for recipient3
        let packet = create_test_packet(source, Some(recipient3));
        queue.store(recipient3, packet).unwrap();

        assert_eq!(queue.total_packets(), 6);
        assert_eq!(queue.recipient_count(), 3);
    }

    #[test]
    fn test_fifo_order() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Store packets with different payloads
        let mut packet_ids = Vec::new();
        for i in 0..5 {
            let mut packet = create_test_packet(source, Some(recipient));
            packet.payload = vec![i];
            packet_ids.push(packet.id);
            queue.store(recipient, packet).unwrap();
        }

        // Retrieve packets
        let packets = queue.retrieve(&recipient);

        // Verify FIFO order
        assert_eq!(packets.len(), 5);
        for (i, packet) in packets.iter().enumerate() {
            assert_eq!(packet.id, packet_ids[i]);
            assert_eq!(packet.payload, vec![i as u8]);
        }
    }

    #[test]
    fn test_get_queue_size_nonexistent_recipient() {
        let queue = StoreForwardQueue::new(10, Duration::from_secs(3600));
        let recipient = Uuid::new_v4();

        assert_eq!(queue.get_queue_size(&recipient), 0);
    }

    #[test]
    fn test_cleanup_with_multiple_recipients() {
        let mut queue = StoreForwardQueue::new(10, Duration::from_secs(1));
        let recipient1 = Uuid::new_v4();
        let recipient2 = Uuid::new_v4();
        let source = Uuid::new_v4();

        // Add old packet for recipient1
        let old_packet = MeshPacket {
            id: Uuid::new_v4(),
            source,
            destination: Some(recipient1),
            ttl: 5,
            payload: vec![1],
            timestamp: SystemTime::now() - Duration::from_secs(5),
        };
        queue.store(recipient1, old_packet).unwrap();

        // Add recent packet for recipient1
        let recent_packet1 = create_test_packet(source, Some(recipient1));
        queue.store(recipient1, recent_packet1).unwrap();

        // Add recent packet for recipient2
        let recent_packet2 = create_test_packet(source, Some(recipient2));
        queue.store(recipient2, recent_packet2).unwrap();

        assert_eq!(queue.total_packets(), 3);
        assert_eq!(queue.recipient_count(), 2);

        // Cleanup should remove only the old packet
        queue.cleanup_expired();

        assert_eq!(queue.get_queue_size(&recipient1), 1);
        assert_eq!(queue.get_queue_size(&recipient2), 1);
        assert_eq!(queue.total_packets(), 2);
        assert_eq!(queue.recipient_count(), 2);
    }
}
