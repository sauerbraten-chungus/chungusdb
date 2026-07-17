use log::{error, info};
use tonic::{Request, Response, Status};

use crate::db::Db;
use crate::models::IncomingPlayer;

// Include the generated code from the proto file
// tonic_build generates a module based on the package name in the proto
pub mod chungusdb {
    tonic::include_proto!("chungusdb");
}

use chungusdb::chungus_db_server::ChungusDb;
use chungusdb::{RecordMatchStatsRequest, RecordMatchStatsResponse};

pub struct ChungusDbImpl {
    db: Db,
}

impl ChungusDbImpl {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl ChungusDb for ChungusDbImpl {
    async fn record_match_stats(
        &self,
        request: Request<RecordMatchStatsRequest>,
    ) -> Result<Response<RecordMatchStatsResponse>, Status> {
        let match_stats = request.into_inner();

        info!(
            "Received match stats for {} players",
            match_stats.player_stats.len()
        );

        let incoming_players: Vec<IncomingPlayer> = match_stats
            .player_stats
            .into_iter()
            .filter_map(|(chungid, stats)| match (uuid::Uuid::parse_str(&chungid)) {
                Ok(chungid) => Some(IncomingPlayer {
                    chungid,
                    name: stats.name,
                    frags: stats.frags,
                    deaths: stats.deaths,
                    accuracy: stats.accuracy as f64,
                    elo: stats.elo,
                }),
                Err(e) => {
                    error!("Invalid UUID converting player starts into IncomingPlayer");
                    None
                }
            })
            .collect();

        // Use the existing database upsert logic
        // self.db
        //     .upsert_batch_players(incoming_players)
        //     .await
        //     .map_err(|e| {
        //         error!("Failed to upsert match stats: {}", e);
        //         Status::internal(format!("Database error: {}", e))
        //     })?;

        self.db
            .process_match_stats(incoming_players)
            .await
            .map_err(|e| {
                error!("Failed to process mathc stats {}", e);
                Status::internal(format!("Database error: {}", e))
            })?;

        info!("Successfully processed match stats");

        let response = RecordMatchStatsResponse {
            message: "Match stats received and processed".to_string(),
        };

        Ok(Response::new(response))
    }
}
