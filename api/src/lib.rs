mod mediator;
mod resolver;
mod provider;

use enum_position::NumIdentity;
use uuid::Uuid;
use rust_decimal::Decimal;
use serde::{ Serialize, Deserialize };
use thiserror::Error;
use std::net::AddrParseError;


pub use resolver::{Message as ResolverMessage, handler as resolver_handler};
pub use mediator::{Message as MediatorMessage, handler as mediator_handler};
pub use provider::{Message as ProviderMessage, handler as provider_handler};
use std::collections::HashMap;

pub type ProviderName = String;
pub type AnswerWeight = HashMap<String,Decimal>;
pub type AnswerFrom = HashMap<String,Vec<ProviderName>>;
pub type OneshotResponse = Result<String,APIError>;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Provider {
    pub name:ProviderName,
    pub doh_url: String,
    pub vote_weight: Decimal
}

#[derive(Error, Debug, NumIdentity)]
pub enum APIError {

    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),


    #[error("error parsing a string into IPv4")]
    IPv4ParseError(#[from] AddrParseError),

    #[error("Parsing failure: {0}")]
    ParseError(String),

    #[error("unsupported response structure from {http_link:?} : {json:?}")]
    UnsupportedResponseStructure{
        json:serde_json::Value,
        http_link:String
    },

    #[error("No response from {0}")]
    NoResponse(String),

    #[error("Accumulated errors: {0:?}")]
    Accumulated(Vec<String>),

    #[error("Contact your nearest operator")]
    SystemError(Uuid,usize),

}