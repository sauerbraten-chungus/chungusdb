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
        let submitted_players = match_stats.player_stats.len();

        info!(
            "event=match_stats_received submitted_players={}",
            submitted_players
        );

        let incoming_players: Vec<IncomingPlayer> = match_stats
            .player_stats
            .into_iter()
            .filter_map(|(chungid, stats)| match uuid::Uuid::parse_str(&chungid) {
                Ok(chungid) => {
                    info!(
                        "event=player_stats_accepted chungid={} frags={} deaths={} accuracy={:.2} elo={}",
                        chungid, stats.frags, stats.deaths, stats.accuracy, stats.elo
                    );
                    Some(IncomingPlayer {
                        chungid,
                        name: stats.name,
                        frags: stats.frags,
                        deaths: stats.deaths,
                        accuracy: stats.accuracy as f64,
                        elo: stats.elo,
                    })
                }
                Err(error) => {
                    error!(
                        "event=player_stats_rejected reason=invalid_chungid chungid={:?} error={}",
                        chungid, error
                    );
                    None
                }
            })
            .collect();
        let accepted_players = incoming_players.len();

        self.db
            .process_match_stats(incoming_players)
            .await
            .map_err(|error| {
                error!(
                    "event=match_stats_failed accepted_players={} error={}",
                    accepted_players, error
                );
                Status::internal(format!("Database error: {}", error))
            })?;

        info!(
            "event=match_stats_processed submitted_players={} accepted_players={}",
            submitted_players, accepted_players
        );

        let response = RecordMatchStatsResponse {
            message: "Match stats received and processed".to_string(),
        };

        Ok(Response::new(response))
    }
}
