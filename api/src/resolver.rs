use uuid::Uuid;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use crate::{ProviderName, APIError, AnswerWeight, AnswerFrom};
use tokio::sync::{broadcast,oneshot};
use tokio::time::Instant;

#[derive(Debug, Clone,Serialize, Deserialize)]
pub enum Message {
    //ResolveHostname(Uuid,String),
    
    ResolvedIP{
        msg_id:Uuid,
        from:ProviderName,
        from_pct:Decimal,
        ip:String
    },
    
    TotalProviders(Uuid,usize),
    
    ResolveError(Uuid,usize)
}


pub async fn handler(id:Uuid,
receiver: broadcast::Receiver<Message>,
//med_sender:mpsc::Sender<MediatorMessage>,
resp_sender:oneshot::Sender<Result<String, APIError>>) {
    log::info!("{}:: launching...", id);

    futures::pin_mut!(receiver);

    let mut total_providers:usize = 0;

    let mut answers_wt = AnswerWeight::new();
    let mut answers_from = AnswerFrom::new();

    let mut agreed_answer = "".to_string();
    let mut agreed_answer_pct = Decimal::from(0);

    let reply_elapsed = Instant::now();

    while let Ok(message) = receiver.recv().await {


        match message {
            // Message::ResolveHostname(msg_id, hostname) => {
            //     if msg_id != id {
            //         log::warn!("{}:: dropping ResolveHostName message from {}",id, msg_id);
            //     } else {
            //         if let Err(e) = med_sender.send(
            //             MediatorMessage::ResolveHostname(msg_id, hostname)
            //         ).await {
            //             log::warn!("{}:: failed to send message to mediator: {:?}", msg_id,e);
            //         }
            //     }
            // }
            
            Message::TotalProviders(msg_id, len) => {
                total_providers = len;

                log::info!("{}:: total providers to wait for: {} from {}",id,total_providers,msg_id);
            }
            
            Message::ResolvedIP { msg_id, from, from_pct, ip } => {
                if msg_id != id {
                    log::warn!("{}:: dropping ResolvedIP message from {}",id, msg_id);
                } else {
                    let answer = ip.to_string();
                    
                    if answer != agreed_answer {
                        let answer_pct = 
                            from_pct + answers_wt.get(&answer).unwrap_or(&Decimal::from(0));
                    
                        if answer_pct > agreed_answer_pct {
                            agreed_answer_pct = answer_pct.clone();
                            agreed_answer = answer.clone();

                            log::info!("{}:: found ip {} @ {}%.", msg_id, answer, answer_pct);
                        }

                        answers_wt.insert(answer.clone(), answer_pct);
                        
                    } else {
                        agreed_answer_pct += from_pct;
                        answers_wt.insert(agreed_answer.clone(), agreed_answer_pct.clone());

                        log::info!("{}:: ip {} @ {}%", msg_id, answer, agreed_answer_pct);
                    }


                    let mut providers = answers_from.get(&answer).unwrap_or(&vec![]).clone();
                    providers.push(from);
                    answers_from.insert(answer, providers);

                    let elapsed_time = reply_elapsed.elapsed().as_millis();
                    log::info!("{}:: elapsed time: {}ms", id, elapsed_time);

                    total_providers -= 1;
                    log::info!("{}:: waiting replies from {} providers...",id, total_providers);

                    if total_providers == 0 || elapsed_time >= 1000 {
                        resp_sender.send(Ok(agreed_answer.clone())).ok();
                        return;
                    }

                }
            }
            
            Message::ResolveError(msg_id, error) => {
                if msg_id != id {
                    log::warn!("{}:: dropping ResolveError message from {}", id, msg_id);
                } else {
                    log::error!("{}:: {:?}", msg_id, error);

                    let elapsed_time = reply_elapsed.elapsed().as_millis();
                    log::info!("{}:: elapsed time: {}ms", id, elapsed_time);

                    total_providers -= 1;
                    log::info!("{}:: waiting replies from {} providers...",id, total_providers);

                    if total_providers == 0 || elapsed_time >= 1000 {
                        if agreed_answer.is_empty() {
                            resp_sender.send(Err(APIError::SystemError(msg_id,error))).ok();
                        } else {
                            resp_sender.send(Ok(agreed_answer.clone())).ok();
                        }

                        return;
                    }

                }
            }
           
        }
    }


}
