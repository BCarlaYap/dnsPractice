use serde::{Serialize,Deserialize};

use std::collections::HashMap;

type ProviderName = String;
type ProviderDOHLink = String;
type ProviderWeight = u32;

#[derive(Serialize, Deserialize)]
pub struct Provider{
    pub name: ProviderName,
    pub doh_link: ProviderDOHLink,
    pub vote_wt: ProviderWeight
}

#[derive(Serialize, Deserialize)]
pub struct AddProvider{
    pub provider:Provider
}


#[derive(Serialize, Deserialize)]
pub struct UpdateQuota(u32);


#[derive(Serialize, Deserialize)]
pub struct UpdateDOHLink{
    pub name:ProviderName,
    pub new_doh_link:ProviderDOHLink
}

#[derive(Serialize, Deserialize)]
pub struct DNSQuery{
    pub query:String
}

#[derive(Serialize, Deserialize)]
pub struct ServerActor{
    pub doh_link_map:HashMap<ProviderName,ProviderDOHLink>,
    pub vote_wt_map: HashMap<ProviderName,ProviderWeight>,
    pub quota: u32
}




#[derive(Serialize, Deserialize)]
pub struct CollatorActor{
    pub data_wt_map: HashMap<String,ProviderWeight>
}

#[derive(Serialize, Deserialize)]
pub struct CollateData{
    pub provider:Provider,
    pub data_json:String
}









