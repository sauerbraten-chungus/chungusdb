use crate::models::{IncomingPlayer, Player};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use uuid::Uuid;

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let url = env::var("DATABASE_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Ok(Db { pool })
    }

    // pub async fn get_player_by_id(&self, id: i64) -> Result<Player, sqlx::Error> {
    //     let player = sqlx::query_as!(Player, "SELECT * FROM players WHERE id = $1", id)
    //         .fetch_one(&self.pool)
    //         .await?;
    //     Ok(player)
    // }

    // pub async fn get_all_players(&self) -> Result<Vec<Player>, sqlx::Error> {
    //     let players = sqlx::query_as!(Player, "SELECT * FROM players")
    //         .fetch_all(&self.pool)
    //         .await?;

    //     Ok(players)
    // }

    pub async fn process_match_stats(
        &self,
        incoming_players: Vec<IncomingPlayer>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        let match_id = Db::insert_match(&mut transaction).await?;
        Db::upsert_batch_players(&mut transaction, &incoming_players).await?;
        Db::insert_batch_match_participants(&mut transaction, &incoming_players, match_id).await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn insert_match(
        transaction: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    ) -> Result<Uuid, sqlx::Error> {
        let row = sqlx::query!(
            r#"
               INSERT INTO matches DEFAULT VALUES
               RETURNING id
            "#
        )
        .fetch_one(&mut **transaction)
        .await?;

        Ok(row.id)
    }

    async fn upsert_batch_players(
        transaction: &mut sqlx::Transaction<'static, sqlx::Postgres>,
        incoming_players: &Vec<IncomingPlayer>,
    ) -> Result<(), sqlx::Error> {
        let chungids: Vec<Uuid> = incoming_players
            .iter()
            .map(|player| player.chungid)
            .collect();
        let names: Vec<String> = incoming_players
            .iter()
            .map(|player| player.name.clone())
            .collect();
        let frags: Vec<i32> = incoming_players.iter().map(|player| player.frags).collect();
        let deaths: Vec<i32> = incoming_players
            .iter()
            .map(|player| player.deaths)
            .collect();
        let accuracies: Vec<f64> = incoming_players
            .iter()
            .map(|player| player.accuracy)
            .collect();
        let elos: Vec<i32> = incoming_players.iter().map(|player| player.elo).collect();

        sqlx::query!(
            r#"
            INSERT INTO players (chungid, name, frags, deaths, accuracy, matches_played, elo)
            SELECT
                t.chungid,
                t.name,
                t.frags,
                t.deaths,
                t.accuracy,
                1 as matches_played, --Dummy value
                t.elo
            FROM UNNEST (
                $1::uuid[],
                $2::text[],
                $3::int[],
                $4::int[],
                $5::float8[],
                $6::int[]
            ) AS t(chungid, name, frags, deaths, accuracy, elo)
            ON CONFLICT (chungid) DO UPDATE
            SET name = EXCLUDED.name,
                frags = players.frags + EXCLUDED.frags,
                deaths = players.deaths + EXCLUDED.deaths,
                accuracy = (players.accuracy * players.matches_played + EXCLUDED.accuracy) / (players.matches_played + 1)::numeric,
                matches_played = players.matches_played + 1,
                elo = players.elo + EXCLUDED.elo,
                updated_at = NOW()
            "#,
            &chungids,
            &names,
            &frags,
            &deaths,
            &accuracies,
            &elos,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    async fn insert_batch_match_participants(
        transaction: &mut sqlx::Transaction<'static, sqlx::Postgres>,
        incoming_players: &Vec<IncomingPlayer>,
        match_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let chungids: Vec<Uuid> = incoming_players
            .iter()
            .map(|player| player.chungid)
            .collect();
        let names: Vec<String> = incoming_players
            .iter()
            .map(|player| player.name.clone())
            .collect();
        let frags: Vec<i32> = incoming_players.iter().map(|player| player.frags).collect();
        let deaths: Vec<i32> = incoming_players
            .iter()
            .map(|player| player.deaths)
            .collect();
        let accuracies: Vec<f64> = incoming_players
            .iter()
            .map(|player| player.accuracy)
            .collect();
        let elos: Vec<i32> = incoming_players.iter().map(|player| player.elo).collect();

        sqlx::query!(
            r#"
              INSERT INTO match_participants (match_id, chungid, name, frags, deaths, accuracy, elo)
              SELECT
                  $1,
                  t.chungid,
                  t.name,
                  t.frags,
                  t.deaths,
                  t.accuracy,
                  t.elo --Currently used wrong, want to use it as a present-of value
              FROM UNNEST (
                  $2::uuid[],
                  $3::text[],
                  $4::int[],
                  $5::int[],
                  $6::float8[],
                  $7::int[]
              ) AS t(chungid, name, frags, deaths, accuracy, elo)
            "#,
            match_id,
            &chungids,
            &names,
            &frags,
            &deaths,
            &accuracies,
            &elos
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
