use radius_sdk::validation::symbiotic::types::map::HashMap;

use super::prelude::*;

pub type IP = String;

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key())]
pub struct MevSearcherInfos(HashMap<IP, Vec<String>>);

impl Default for MevSearcherInfos {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl MevSearcherInfos {
    /// Check if a given IP exists
    pub fn contains_ip(&self, ip: &str) -> bool {
        self.0.contains_key(ip)
    }

    /// Check if a given rollup_id exists for an IP
    pub fn contains_rollup_id(&self, ip: &str, rollup_id: &RollupId) -> bool {
        self.0
            .get(ip)
            .map_or(false, |rollups| rollups.iter().any(|id| id == rollup_id))
    }

    /// Add a rollup_id to an IP entry (create if doesn't exist)
    pub fn add_rollup_id(&mut self, ip: &str, rollup_id: &RollupId) {
        self.0
            .entry(ip.to_string())
            .or_insert_with(Vec::new)
            .push(rollup_id.to_string());
    }

    /// Remove a rollup_id from an IP entry, and clean up if empty
    pub fn remove_rollup_id(&mut self, ip: &str, rollup_id: &RollupId) {
        if let Some(rollups) = self.0.get_mut(ip) {
            rollups.retain(|id| id != rollup_id);
            if rollups.is_empty() {
                self.0.remove(ip);
            }
        }
    }

    /// Replace or insert rollup_id list for an IP
    pub fn insert(&mut self, ip: IP, rollup_ids: Vec<String>) -> Option<Vec<String>> {
        self.0.insert(ip, rollup_ids)
    }

    /// Remove an IP entry entirely
    pub fn remove(&mut self, ip: &IP) -> Option<Vec<String>> {
        self.0.remove(ip)
    }

    /// Get all IPs that contain a given rollup_id
    pub fn get_ip_list_by_rollup_id(&self, rollup_id: &RollupId) -> Vec<IP> {
        self.0
            .iter()
            .filter_map(|(ip, rollups)| {
                if rollups.iter().any(|id| id == rollup_id) {
                    Some(ip.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
