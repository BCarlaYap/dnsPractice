
use once_cell::sync::OnceCell;
use tokio_postgres::{Client, NoTls};

pub type Error = tokio_postgres::error::Error;

static PG_CLIENT: OnceCell<Client> = OnceCell::new();

pub fn set_client(client:Client) {
    PG_CLIENT.set(client).expect("PG_CLIENT already setup");
}

pub fn get_client() -> &'static Client { PG_CLIENT.get().unwrap() }


pub async fn connect() -> Result<(), Error> {
    let host = std::env::var("POSTGRES.HOST").expect("POSTGRES.HOST is not set.");
    let pass = std::env::var("POSTGRES.PASS").expect("POSTGRES.PASS is not set.");
    let port = std::env::var("POSTGRES.PORT").expect("POSTGRES.PORT is not set.");

    log::info!("host:{} pass:{} port:{}", host,pass,port);

    let pg_connect = format!("user=postgres password={} host={}",pass,host);

     tokio_postgres::connect(&pg_connect, NoTls).await.map(|(client, connection)| {
         set_client(client);
         log::info!("connect successful.");

         tokio::spawn( async move {
             if let Err(e) = connection.await {
                 log::error!("connection error: {}",e );
             }
         });
         ()
    })
}


pub async fn create_table(create_table_sql:&str) -> Result<(),Error> {
    let client = get_client();
    log::info!("creating a table...");

    client.simple_query(create_table_sql).await.map(|_| ())
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
