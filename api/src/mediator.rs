use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::{Provider, ResolverMessage, ProviderMessage, ProviderName, provider_handler};

use tokio::sync::{broadcast, mpsc};
use rust_decimal::Decimal;
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message{
    AddProvider(Uuid,Provider),
    ResolveHostname(Uuid, String)
}

pub async fn handler() -> (mpsc::Sender<Message>, broadcast::Sender<ResolverMessage>) {
    log::info!("launching...");

    let mut providers_map:HashMap<ProviderName, mpsc::Sender<ProviderMessage>> = HashMap::new();

    let (sender, receiver) = mpsc::channel::<Message>(100);
    let (resolver_sender, _)= broadcast::channel::<ResolverMessage>(100);


    let resolver_clone = resolver_sender.clone();

    tokio::spawn( async move {
        futures::pin_mut!(receiver);

        let mut actual_sum:Decimal = Decimal::new(0,0);
        let mut actual_providers:usize = 0;

        while let Some(message) = receiver.recv().await {
            match message {
                Message::AddProvider(msg_id, provider) => {
                    actual_sum += provider.vote_weight;
                    actual_providers += 1;

                    log::info!("{}:: Total Providers:{} Vote Sum:{}", msg_id, actual_providers, actual_sum);

                    let (provider_sender, provider_receiver) =
                        mpsc::channel::<ProviderMessage>(100);

                    providers_map.insert(provider.name.clone(), provider_sender);

                    for (provider, prov_sender) in providers_map.iter() {
                        let msg = ProviderMessage::AddToTotalSum(msg_id.clone(), actual_sum.clone());

                        if let Err(e) = prov_sender.send(msg).await {
                            log::warn!("{}:: failed to send message to {}: {:?}", msg_id, provider, e);
                        };
                    }


                    tokio::spawn( provider_handler(provider, provider_receiver,resolver_clone.clone()));

                }

                Message::ResolveHostname(msg_id, hostname) => {
                    resolver_clone.send(ResolverMessage::TotalProviders(msg_id,actual_providers)).ok();

                    for (provider, sender) in providers_map.iter() {
                       let msg = ProviderMessage::ResolveHostname(msg_id.clone(),hostname.clone());

                       if let Err(e) = sender.send(msg).await {
                           log::error!("{}:: failed to send message to {}: {:?}", msg_id,provider,e);

                       }

                    }
                }
            }
        }
    });

    (sender, resolver_sender)
}