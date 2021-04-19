use uuid::Uuid;
use rust_decimal::Decimal;
use crate::{APIError, Provider, ResolverMessage};
use serde_json::Value as SerdeValue;
use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::{Serialize,Deserialize};
use tokio::sync::{mpsc, broadcast};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    AddToTotalSum(Uuid,Decimal),
    ResolveHostname(Uuid,String)
}


async fn http_get_reqwest(http_link:&str) -> Result<String,APIError> {
    let client = reqwest::Client::new();

    let response:SerdeValue = client.get(http_link)
        .header(reqwest::header::ACCEPT, "application/dns-json")
        .send().await.map_err(APIError::ReqwestError)?
        .json().await.map_err( APIError::ReqwestError)?;


    fn extract_address(addr:&SerdeValue) -> Result<String,APIError> {
        if let Some(addr)  = addr.as_str(){
            Ipv4Addr::from_str(addr)
                .map(|addr| { addr.to_string() })
                .map_err(|err| APIError::IPv4ParseError(err) )
        } else {
            Err(APIError::ParseError(format!("Cannot format to string: {:?}", addr)))
        }
    }


    match &response["Answer"] {
        SerdeValue::String(response) => Ok(response.clone()),

        SerdeValue::Object(response) =>
            if let Some(addr) = response.get("data") {
                extract_address(addr)
            } else {
                Err(APIError::ParseError(format!("Not found `data` field: {:?}",response)))
            },

        SerdeValue::Array(response) => {
            let mut acc_err:Vec<String> = vec![];
            let mut answer:Result<String,_> = Err(APIError::NoResponse(format!("no actual response: {:?}",response)));

            for response in response {
                match extract_address(&response["data"]) {
                    Ok(addr) => { answer = Ok(addr); }
                    Err(err) => {
                        acc_err.push(format!("{}",err));
                    }
                }
            }

            if answer.is_err() && !acc_err.is_empty() {
                answer = Err(APIError::Accumulated(acc_err));
            }

            answer
        }

        _ => {
            Err(APIError::UnsupportedResponseStructure {
                json: response,
                http_link: http_link.clone().parse().unwrap()
            })
        }
    }
}


pub async fn handler(provider:Provider,
                     receiver: mpsc::Receiver<Message>, 
                     resolver_sender: broadcast::Sender<ResolverMessage>) {
    log::info!("{}:: launching...", &provider.name);

    futures::pin_mut!(receiver);

    // first time initialization
    let mut actual_sum = provider.vote_weight.clone();
    let mut actual_weight = actual_sum.clone();

    while let Some(message) = receiver.recv().await {
        match message {
            Message::AddToTotalSum(msg_id, updated_sum) => {
                actual_sum = updated_sum;

                actual_weight = (&provider.vote_weight/actual_sum) * Decimal::from(100);

                log::info!("{}:: {}:: new vote weight: {} @ sum: {}",
                           msg_id, &provider.name, actual_weight, actual_sum);
            }

            Message::ResolveHostname(msg_id, hostname) => {
                let http_link = format!("{}?name={}", provider.doh_url, hostname);
                log::info!("{}:: {}:: request: {}", msg_id, &provider.name, http_link);

                match http_get_reqwest(&http_link).await {
                    Ok(ip) => {
                        resolver_sender.send(
                            ResolverMessage::ResolvedIP {
                                msg_id,
                                from: provider.name.clone(),
                                from_pct: actual_weight.clone(),
                                ip
                            }).ok();
                    }

                    Err(err) => {
                        log::error!("{}:: {}:: failed to resolve: {:?}", msg_id, &provider.name,err);

                        resolver_sender.send(
                            ResolverMessage::ResolveError(msg_id, err.position())
                        ).ok();
                    }
                }
            }
        }
    }
}
