use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossbeam::channel::{select, unbounded, Receiver};

use ap_core::match_insights::MatchInsightDB;
use ap_core::processor::{ArenaEventSource, PlayerLogProcessor};
use ap_core::replay::MatchReplayBuilder;
use ap_core::storage_backends::{ArenaMatchStorageBackend, DirectoryStorageBackend};


#[derive(Debug, Parser)]
#[command(about = "Tries to scrape useful data from mtga detailed logs")]
struct Args {
    #[arg(short, long, help = "Location of Player.log file")]
    player_log: PathBuf,
    #[arg(short, long, help = "directory to write replay output files")]
    output_dir: Option<PathBuf>,
    #[arg(short, long, help = "database to write match data to")]
    db: Option<PathBuf>,
    #[arg(short, long, help = "database of cards to reference")]
    cards_db: Option<PathBuf>,
    #[arg(long, action = clap::ArgAction::SetTrue, help = "enable debug logging")]
    debug: bool,
    #[arg(
        short, long, action = clap::ArgAction::SetTrue, help = "wait for new events on Player.log, useful if you are actively playing MTGA"
    )]
    follow: bool,
}

fn ctrl_c_channel() -> Result<Receiver<()>> {
    let (ctrl_c_tx, ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        ctrl_c_tx.send(()).unwrap_or(());
    })?;
    Ok(ctrl_c_rx)
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;
    tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    let mut processor = PlayerLogProcessor::try_new(args.player_log)?;
    let mut match_replay_builder = MatchReplayBuilder::new();
    let mut storage_backends: Vec<Box<dyn ArenaMatchStorageBackend>> = Vec::new();
    let cards_db =
        ap_core::cards::CardsDatabase::new(args.cards_db.unwrap_or("data/cards-full.json".into()))?;
    let follow = args.follow;

    let ctrl_c_rx = ctrl_c_channel()?;
    if let Some(output_dir) = args.output_dir {
        std::fs::create_dir_all(&output_dir)?;
        storage_backends.push(Box::new(DirectoryStorageBackend::new(output_dir)));
    }

    if let Some(db_path) = args.db {
        let conn = rusqlite::Connection::open(db_path)?;
        let mut db = MatchInsightDB::new(conn, cards_db);
        db.init()?;
        storage_backends.push(Box::new(db));
    }


    loop {
        select! {
            recv(ctrl_c_rx) -> _ => {
                break;
            }
            // notify crate doesn't fully capture fs events like I want it to
            default(Duration::from_secs(1)) => {
                while let Some(parse_output) = processor.get_next_event() {
                    if match_replay_builder.ingest_event(parse_output) {

                        let match_replay = match_replay_builder.build()?;
                        for backend in &mut storage_backends {
                            backend.write(&match_replay)?;
                        }
                        match_replay_builder = MatchReplayBuilder::new();
                    }
                }
                if !follow {
                    break;
                }
            }
        }
    }

    Ok(())
}
