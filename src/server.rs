


use crate::models::{Provider, AddProvider, ServerActor};
use actix::Actor;
use std::io;
use actix::prelude::*;
use std::collections::HashMap;


pub struct GetAllProviders;
impl Message for GetAllProviders {
    type Result =  Result<Vec<Provider>,io::Error>;
}

impl Handler<GetAllProviders> for ServerActor{
    type Result = Result<Vec<Provider>,io::Error>;

    fn handle(&mut self, _: GetAllProviders, _: &mut Self::Context) -> Self::Result {
        println!("received GetAllProvider");
        let mut provider_list: Vec<Provider> = Vec::new();

        for(key, vote_weight) in self.vote_wt_map.iter() {
            let provider = Provider{
                name: key.clone(),
                doh_link: self.doh_link_map.get(key).unwrap().to_string(),
                vote_wt: vote_weight.clone()
            };

            provider_list.push(provider);
        };

        Ok(provider_list)
    }
}


pub struct GetQuotaProviders;
impl Message for GetQuotaProviders {
    type Result =  Result<Vec<Provider>,io::Error>;

}

impl Handler<GetQuotaProviders> for ServerActor {
    type Result =  Result<Vec<Provider>,io::Error>;

    fn handle(&mut self, _: GetQuotaProviders, _: &mut Self::Context) -> Self::Result {
        println!("received GetQuotaProvider");

        let mut all_providers = self.vote_wt_map.clone().into_iter().collect::<Vec<(String,u32)>>();

        all_providers.sort_by(|(_,first), (_,second)| second.cmp(first));

        let consumed_quota:u32 = 0;
        let mut quota_providers:Vec<Provider> = Vec::new();

        for elem in all_providers {
            if consumed_quota >= self.quota { break; }

            quota_providers.push(Provider{
                name: elem.0.clone(),
                doh_link: self.doh_link_map.get(&elem.0).unwrap().to_string(),
                vote_wt: elem.1
            });
        };

        Ok(quota_providers)


    }
}



impl Message for AddProvider{
    type Result = Result<String, io::Error>;
}

impl Handler<AddProvider> for ServerActor{
    type Result = Result<String, io::Error>;

    fn handle(&mut self, msg: AddProvider, _: &mut Self::Context) -> Self::Result {
        println!("received AddProvider: {}",&msg.provider.name);
        if self.quota <= msg.provider.vote_wt {
            println!("dictatorship on provider {}, at weight {}, when quota is just {}", &msg.provider.name, &msg.provider.vote_wt, &self.quota);
        }

        if self.doh_link_map.contains_key(&msg.provider.name) {
            println!("updating provider's {} old values. Current doh_link:{} current weight: {}", &msg.provider.name, &msg.provider.doh_link, &msg.provider.vote_wt);
        }

       self.vote_wt_map.insert(msg.provider.name.clone(),msg.provider.vote_wt);
        self.doh_link_map.insert(msg.provider.name.clone(),msg.provider.doh_link);
        Ok("ok".to_string())




    }
}


impl Actor for ServerActor{
    type Context = Context<Self>;
}
 impl ServerActor{
    pub fn new() -> Self {
        Self{
            vote_wt_map: HashMap::new(),
            doh_link_map: HashMap::new(),
            quota:99
        }
    }

}

