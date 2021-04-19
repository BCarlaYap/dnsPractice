
use dotenv::dotenv;
use tokio::sync::{mpsc, broadcast, oneshot};
use api_lib::{MediatorMessage, mediator_handler};
use postgres_helper::{create_table, connect};
use std::path::Path;
use crate::handler::{get_all_providers, resolve_hostname, create_provider};
use uuid::Uuid;
use warp::Filter;
use std::convert::Infallible;


mod handler {
    use super::*;
    use api_lib::{Provider, APIError, ResolverMessage, resolver_handler};
    use postgres_helper::get_client;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use uuid::Uuid;
    use warp::{Reply,Rejection};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct NameQuery {
        pub name: Option<String>,
    }

    pub type OneshotResponse = Result<String,APIError>;

    pub async fn get_all_providers() -> Vec<Provider> {
        let client = get_client();

        let select = format!("SELECT {} FROM {}", FIELDS, TABLE_NAME);

        match client.query(select.as_str(), &[]).await {
            Ok(rows) => {
                rows.iter().map(|row| {
                    Provider {
                        name: row.get(0),
                        doh_url: row.get(1),
                        vote_weight: Decimal::from_f32(row.get(2)).unwrap()
                    }
                }).collect()
            }

            Err(e) => {
                log::error!("get_all_providers query failed: {:?}",e);

                vec![]
            }
        }
    }

    pub async fn create_provider(provider:Provider, med_sender: mpsc::Sender<MediatorMessage>)
    -> Result<impl Reply,Rejection> {
        let client = get_client();

        let insert  = format!("INSERT INTO {} ( {} ) VALUES ( $1, $2, $3)", TABLE_NAME, FIELDS);

        match client.query(insert.as_str(),
                           &[&provider.name,
                               &provider.doh_url,
                               &provider.vote_weight.to_f32().unwrap()]
        ).await {
            Ok(_) => {
                let msg_id = Uuid::new_v4();

                let provider_name = provider.name.clone();
                log::info!("{}:: {}:: creating provider...", msg_id, provider_name);

                if let Err(e) =
                med_sender.send( MediatorMessage::AddProvider(msg_id.clone(), provider)).await {
                    log::error!("{}:: {}:: {:?}",msg_id, provider_name, e);

                    Ok::<_, Rejection>( warp::reply::with_status(
                        "Could not add provider. Contact Sys Admin.",
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR
                    ))
                } else {
                    Ok::<_,Rejection>( warp::reply::with_status(
                        "Added provider to the list.",
                        warp::http::StatusCode::CREATED
                    ))
                }
            }

            Err(e) => {
                log::error!("{}:: {:?}", provider.name, e);

                Ok::<_,Rejection>( warp::reply::with_status(
                    "DBQUery error. Contact Sys Admin.",
                    warp::http::StatusCode::BAD_REQUEST
                ))
            }
        }

    }


    pub async fn resolve_hostname(query: NameQuery,
                                  med_sender: mpsc::Sender<MediatorMessage>,
                                  receiver:broadcast::Receiver<ResolverMessage>
    ) -> String {
        let (tx, rx) = oneshot::channel::<OneshotResponse>();

        let msg_id = Uuid::new_v4();

        tokio::spawn(resolver_handler(msg_id.clone(),receiver,tx));

        if let Err(e) = med_sender.send(
            MediatorMessage::ResolveHostname(msg_id.clone(),query.name.unwrap())
        ).await {
            log::error!("{}:: failed to send message to mediator, {:?}", msg_id,e);
        };

        match rx.await {
            Ok(reply) => {
                match reply {
                    Ok(reply) => { reply }
                    Err(e) => {
                        let err = format!("{:?}",e);
                        log::error!("{}:: {}", msg_id, err);
                        err
                    }
                }
            }

            Err(e) => {
                let err = format!("{:?}",e);
                log::error!("{}:: {}", msg_id,err);
                err
            }
        }
    }
}


async fn init(med_sender: &mpsc::Sender<MediatorMessage>) {
    let sql_file_path = std::env::var("POSTGRES.SCRIPT_PATH").expect("POSTGRES.SCRIPT_PATH is not set.");

    let sql_file_path = Path::new(&sql_file_path);

    let create_table_script = std::fs::read_to_string(sql_file_path).unwrap();

    if let Err(e) = create_table(create_table_script.as_str()).await {
        log::error!("failed to initialize the table: {:?}",e);
    } else {
        log::info!("table created");
        let providers = get_all_providers().await;

        for provider in providers.into_iter() {
            let msg_id = Uuid::new_v4();

            log::info!("{}:: broadcasting new provider...", msg_id);

            if let Err(e) = med_sender.send(
                MediatorMessage::AddProvider(msg_id.clone(), provider)
            ).await {
                log::warn!("{}:: failed to send AddProvider message to mediator: {:?}", msg_id, e);
            }
        }
    }
}

pub fn logging() {
    use log4rs::{
        encode::pattern::PatternEncoder,
        filter::threshold::ThresholdFilter,

        config::{Appender, Root},
        append::{
            file::FileAppender,
            console::{ConsoleAppender, Target},
        }
    };

    use log::LevelFilter;


    let stdout = ConsoleAppender::builder().target(Target::Stdout).build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {h({l})} {M} - {m}{n}")))
        .build("log/api.log")
        .unwrap();

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(requests)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log::LevelFilter::Info)))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stdout")
                .build(LevelFilter::Info),
        )
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}

fn with_med_sender(med_sender: mpsc::Sender<MediatorMessage>)
                   -> impl Filter<Extract=(mpsc::Sender<MediatorMessage>, ), Error=Infallible> + Clone {
    warp::any().map(move || med_sender.clone())
}

pub static FIELDS: &str = "name, doh_url, vote_weight";
pub static TABLE_NAME:&str = "providers";

#[tokio::main]
async fn main() {
    dotenv().ok();
    logging();


    let (med_sender, resolver_sender) = mediator_handler().await;

    match connect().await {
        Ok(_) => {
            init(&med_sender).await;

            // resolver path
            let resolver_sender_clone = resolver_sender.clone();
            let resolver = warp::get()
                .and(warp::path("resolve"))
                .and(warp::query())
                .and(with_med_sender(med_sender.clone()))
                .and(warp::any().map(move || resolver_sender_clone.subscribe()))
                .and_then(|x,y,z| async move {
                    let response = resolve_hostname(x,y,z).await;

                    Ok::<_,warp::Rejection>(warp::reply::json(&response))
                });


            // add provider path
            let add_provider = warp::post()
                .and(warp::path("provider"))
                .and(warp::path::end())
                .and(warp::body::content_length_limit(1024*16)
                    .and(warp::body::json()))
                .and(with_med_sender(med_sender.clone()))
                .and_then(create_provider);

            // get providers list path
            let get_all_provider = warp::get()
                .and(warp::path("providers"))
                .and( warp::path::end())
                .and_then(|| async move {
                    let providers = get_all_providers().await;

                    Ok::<_,warp::Rejection>(warp::reply::json(&providers))
                });

            warp::serve(resolver
                .or(add_provider)
                .or(get_all_provider)
            ).run(([127,0,0,1], 3030))
                .await;
        }

        Err(e) => {
            log::error!("postgres connect issues: {:?}",e);
        }
    }

    println!("Hello, world!");

}
