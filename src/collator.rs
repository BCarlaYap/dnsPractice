

use crate::models::{CollateData, CollatorActor};
use actix::Actor;
use actix::prelude::*;
use std::collections::HashMap;
use serde_json::Value;


pub struct GetExactData;
impl Message for GetExactData {
    type Result = String;
}

impl Handler<GetExactData> for CollatorActor {
    type Result = String;

    fn handle(&mut self, _:GetExactData, _: &mut Self::Context) -> Self::Result {
        println!("received GetExactData");

        let mut fin_wt:u32 = 0;
        let mut fin_data:String = "".to_string();

        for (extracted_data, wt) in self.data_wt_map.iter() {
            println!("{} has weight: {}", extracted_data,wt);
            if wt > &fin_wt {
                fin_wt = wt.clone();
                fin_data = extracted_data.clone();

                println!("new data {} with weight {}",&fin_data,&fin_wt);
            }
        };

        fin_data


    }


}


impl Message for CollateData {
    type Result = bool;
}

impl Handler<CollateData> for CollatorActor {
    type Result = bool;

    fn handle(&mut self, msg:CollateData, _: &mut Self::Context) -> Self::Result {
        println!("received CollateData: {},{}", &msg.provider.name, &msg.data_json);

        let v:Value = serde_json::from_str(msg.data_json.as_str()).unwrap();

        if v.is_null() {
            return false
        }
        else if v.is_array() {

           let value_arr =  v.as_array().unwrap();

            for elem in value_arr {

                let extracted_val = elem["data"].as_str().unwrap();


                let cur_wt = self.data_wt_map.get(extracted_val).unwrap_or(&0);
                println!("save data {} with weight {} + previous {}", extracted_val,cur_wt, &msg.provider.vote_wt);

                self.data_wt_map.insert(extracted_val.to_string(),cur_wt+msg.provider.vote_wt);

            }
        } else {

            let extracted_val = v["data"].as_str().unwrap();

            println!("save data {} with weight {}", extracted_val,&msg.provider.vote_wt);

            self.data_wt_map.insert(extracted_val.to_string(),msg.provider.vote_wt);
        }

        true
    }

}

impl Actor for CollatorActor{
    type Context = Context<Self>;
}

impl CollatorActor {
    pub fn new() -> Self {
        Self{
            data_wt_map: HashMap::new()
        }
    }
}

