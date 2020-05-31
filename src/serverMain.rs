mod models;
mod config;
mod handlers;
mod server;
mod collator;

use actix_web::{HttpServer, App,web, middleware};

use std::io;
use dotenv::dotenv;
use crate::handlers::*;
use actix::Actor;


#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
   // let pool = config.pg.create_pool(NoTls).unwrap();

    let server_addr = models::ServerActor::new().start();


    println!("START THE MAIN http://{}:{}",config.server.host, config.server.port);
    HttpServer::new(move||{
        App::new()
            .data(server_addr.clone())

            .wrap(middleware::Logger::default())
            .route("/provider{_:/?}", web::get().to(get_all_provider))
            .route("/provider{_:/?}", web::post().to(add_provider))
           .route("/resolve{_:/?}", web::get().to(resolver))
            })
        .bind(format!("{}:{}", config.server.host,config.server.port))?
        .run()
        .await;
    Ok(())
}
