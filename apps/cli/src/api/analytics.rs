use crate::api::client::NoteClient;
use crate::error::Result;
use crate::models::Stats;
use serde_json::Value;

impl NoteClient {
    /// Get page view statistics
    #[allow(dead_code)]
    pub async fn get_stats(&self, filter: Option<&str>, page: u32) -> Result<Vec<Stats>> {
        let mut path = format!("/api/v1/stats/pv?page={page}");

        if let Some(f) = filter {
            path = format!("{path}&filter={f}");
        }

        let response = self.get(&path).await?;
        let json: Value = response.json().await?;

        if let Some(data) = json.get("data") {
            // Parse the stats data
            let mut stats_list = Vec::new();

            if let Some(items) = data.as_array() {
                for item in items {
                    let stat = Stats {
                        pv: item.get("pv").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        read: item.get("read").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        like: item.get("like").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        comment: item.get("comment").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    };
                    stats_list.push(stat);
                }
            }

            Ok(stats_list)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get notification counts
    #[allow(dead_code)]
    pub async fn get_notice_counts(&self) -> Result<u32> {
        let path = "/api/v3/notice_counts";
        let response = self.get(path).await?;
        let json: Value = response.json().await?;

        if let Some(count) = json
            .get("data")
            .and_then(|d| d.get("count"))
            .and_then(|c| c.as_u64())
        {
            Ok(count as u32)
        } else {
            Ok(0)
        }
    }
}
