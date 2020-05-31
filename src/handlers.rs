extern crate futures;

use crate::models::{AddProvider, CollatorActor, CollateData};
use crate::collator::*;

use actix_web::{web, Responder, HttpResponse, HttpRequest, ResponseError};

use actix::{Addr, Actor, MailboxError};
use crate::server::{GetAllProviders,GetQuotaProviders};
use crate::models::{Provider,ServerActor};
use serde_json::Value;


pub async fn get_all_provider(_:HttpRequest,
                              server_data: web::Data<Addr<ServerActor>>)  -> Result<impl Responder, MailboxError> {
   server_data.get_ref().send(GetAllProviders).await.map(|res|{
        match res {
            Ok(provider_vec) => HttpResponse::Ok().json(provider_vec),
            Err(_) => HttpResponse::Ok().json("list is empty")
        }
    })

}

pub async fn add_provider(_: HttpRequest,
server_data: web::Data<Addr<ServerActor>>,
json: web::Json<Provider>) -> impl Responder {
    println!("received internal command {}",&json.name);
    server_data.get_ref().send(AddProvider{ provider:json.0}).await.map(|res|{
       match res  {
           Ok(send_resp) => HttpResponse::Ok().json(send_resp),
           Err(e) =>  e.error_response()
       }
   })
}



// async fn get_http_actix(query:&str, doh_link:&String) -> impl Responder {
//     let httplink = format!("{}?{}",doh_link,query);
//
//     println!("httplnk: {}",httplink);
//
//     let res = awc::Client::new().get(httplink)
//         .header("Accept","application/dns-json")
//         .header("User-Agent", "Actix-web")
//         .send().await;
//
//
//     match res {
//         Ok(mut x) => HttpResponse::Ok().body(x.body().await.unwrap()),
//         Err(e) => HttpResponse::Ok().json("ERROR")
//     }
//
//
// }



pub async fn resolver(req:HttpRequest, server_data: web::Data<Addr<ServerActor>>) -> impl Responder {
    println!("CARLA CARLA RESOLVERRRRRR!");

    let server_response = server_data.get_ref().send(GetQuotaProviders).await.unwrap();
    let collator_actor = CollatorActor::new().start();

    for prov in server_response.unwrap() {
        let json = get_http_reqwest(req.query_string(),&prov.doh_link).await;

        collator_actor.send(CollateData{
            provider : prov,
            data_json: json
        }).await;
    };

  match collator_actor.send(GetExactData).await {
      Ok(x) =>  HttpResponse::Ok().json(x),
      Err(_) => HttpResponse::Ok().json("nope")
  }

}





async fn get_http_reqwest(query:&str, doh_link:&String) -> String {
    let http_link = format!("{}?{}",doh_link,query);
    println!("http_link reqwest: {}",&http_link);

    let client = reqwest::Client::new();
    let res = client.get(&http_link)
        .header(reqwest::header::ACCEPT,"application/dns-json")
        .send().await.expect("response from dns")
        .text().await.expect("text conversion");


    let v:Value = serde_json::from_str(res.as_str()).unwrap();

    format!("{}",v["Answer"])

}

// async fn requests_in_sequence<'a>(query:&'a str,vals: Vec<Provider>) -> impl Stream<Item =String> {
//     // stream::unfold(vals.into_iter(), |mut vals| async {
//     //     let val = vals.next()?;
//     //     let response = get_http_reqwest(query,&val.doh_link).await;
//     //     Some((response, vals))
//     // })
//
//     stream::unfold(vals.into_iter(), |mut v| async {
//        // let q = query.clone();
//         let haha = v.next().unwrap();
//
//         let data = get_http_reqwest(query.borrow(),&haha.doh_link).await;
//
//         Some((data,v))
//
//     })
// }
//
//
// pub async fn resolver<'a>(req:HttpRequest, server_data: web::Data<Addr<ServerActor>>) -> impl Responder {
//
//     let hello = server_data.get_ref().send(GetAllProvider).await.unwrap();
//     let provider_list = hello.unwrap();
//     let cloner:&'a str = req.query_string();
//
//
//
//     let newResult = executor::block_on(async {
//         let mut s = requests_in_sequence(cloner,provider_list);
//
//         let result = s.await.collect::<Vec<String>>();
//
//         // println!("CARLA CARLA HAHA {}",result[0]);
//         HttpResponse::Ok().json("HAHAHAHAHAHA")
//     });
//
//     newResult
//
// }





