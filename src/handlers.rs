use crate::{app_data::AppData, error::XProtocolError};
use actix_web::{web, HttpResponse};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use web3::{
    ethabi::ParamType,
    signing::{Key, SecretKeyRef},
    types::{H160, H256},
};
use web3_macros::SignV4;

#[derive(SignV4)]
#[primary_type]
#[domain_712("Fomo3d", "1")]
pub struct VaultBuy {
    #[web3_type("ParamType::Uint(256)")]
    pub buy_num: u64,
    #[web3_type("ParamType::Uint(256)")]
    pub team: u64,
    #[web3_type("ParamType::Uint(256)")]
    pub rounds: u64,
    #[web3_type("ParamType::Address")]
    pub account: H160,
    #[web3_type("ParamType::Uint(256)")]
    pub nonce: u64,
}

#[derive(SignV4)]
#[primary_type]
#[domain_712("Fomo3d", "1")]
pub struct Claim {
    #[web3_type("ParamType::Address")]
    pub account: H160,
    #[web3_type("ParamType::Uint(256)")]
    pub number: u64,
    #[web3_type("ParamType::Uint(256)")]
    pub nonce: u64,
}
#[derive(Serialize, Deserialize)]
pub struct MaySignature {
    pub buy_num: u64,
    pub team: u64,
    pub rounds: u64,
    pub account: String,
    pub nonce: u64,
    pub v: u64,
    pub r: H256,
    pub s: H256,
}

#[derive(Serialize, Deserialize)]
pub struct MaySignature2 {
    pub account: String,
    pub number: u64,
    pub nonce: u64,
    pub v: u64,
    pub r: H256,
    pub s: H256,
}
pub struct Handlers;

impl Handlers {
    pub fn app_config(cfg: &mut web::ServiceConfig) {
        cfg.route("/", web::get().to(Self::index))
            .route(
                "/sign/{chain_id}/{buy_num}/{team}/{rounds}/{address}/{nonce}",
                web::get().to(Self::sign),
            )
            .route(
                "/sign2/{chain_id}/{address}/{number}/{nonce}",
                web::get().to(Self::sign2),
            );
    }

    pub async fn index() -> Result<HttpResponse, XProtocolError> {
        Ok(HttpResponse::Ok().body("Hello World"))
    }

    pub async fn sign(
        path: web::Path<(String, u64, u64, u64, String, u64)>,
        data: web::Data<AppData>,
    ) -> Result<HttpResponse, XProtocolError> {
        let (chain_id, buy_num, team, rounds, address, nonce) = path.into_inner();
        let number = data.get_nonce(rounds.clone(), address.clone()).await;
        let address = address.to_lowercase();
        let account = address
            .parse()
            .map_err(|_| XProtocolError::ExpectationFailed)?;
        let contract = "0x5fbdb2315678afecb367f032d93f642f64180aa3";
        println!("contract {:?}", contract);
        let contract = H160::from_str(contract).map_err(|_| XProtocolError::InternalServerError)?;
        println!("contract {:?}", contract);
        let sign: [u8; 32] = VaultBuy {
            buy_num,
            team,
            rounds,
            account,
            nonce,
        }
        .sign_hash(&chain_id, contract)
        .map_err(|_| XProtocolError::InternalServerError)?;

        let secret = SecretKey::from_slice(data.private_key.as_bytes()).unwrap();
        let secret_ref = SecretKeyRef::new(&secret);

        let signature = secret_ref
            .sign(&sign, None)
            .map_err(|_| XProtocolError::InternalServerError)?;
        Ok(HttpResponse::Ok().json(MaySignature {
            buy_num,
            team,
            rounds,
            account: address,
            nonce,
            r: signature.r,
            s: signature.s,
            v: signature.v,
        }))
    }

    pub async fn sign2(
        path: web::Path<(String, String, u64, u64)>,
        data: web::Data<AppData>,
    ) -> Result<HttpResponse, XProtocolError> {
        let (chain_id, address, number, nonce) = path.into_inner();
        let address = address.to_lowercase();
        let account = address
            .parse()
            .map_err(|_| XProtocolError::ExpectationFailed)?;
        let contract = "0x5fbdb2315678afecb367f032d93f642f64180aa3";
        let contract = H160::from_str(contract).map_err(|_| XProtocolError::InternalServerError)?;
        let sign: [u8; 32] = Claim {
            account,
            number,
            nonce,
        }
        .sign_hash(&chain_id, contract)
        .map_err(|_| XProtocolError::InternalServerError)?;

        let secret = SecretKey::from_slice(data.private_key.as_bytes()).unwrap();
        let secret_ref = SecretKeyRef::new(&secret);

        let signature = secret_ref
            .sign(&sign, None)
            .map_err(|_| XProtocolError::InternalServerError)?;
        Ok(HttpResponse::Ok().json(MaySignature2 {
            account: address,
            number,
            nonce,
            r: signature.r,
            s: signature.s,
            v: signature.v,
        }))
    }
}
