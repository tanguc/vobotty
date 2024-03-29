pub mod engine;
pub mod vpn;
pub mod account_repository;

extern crate regex;
extern crate chrono;
extern crate rusoto_dynamodb;
extern crate rusoto_core;

#[macro_use]
extern crate serde;
extern crate serde_json;

use crate::engine::EngineSession;
use crate::engine::amakna::AmaknaBotEngine;
use crate::engine::EngineExecutor;
use crate::account_repository::AccountRepository;

fn main() {
    let account = engine::Account::new(
        "fakeAccount".to_string(),
        "fakeAccountPass".to_string());

    let vpn_account = vpn::VpnCredentials::new(
        "fakeAccount".to_string(),
        "fakeAccountPass".to_string());

    let vpn =
        vpn::Vpn::new(vpn_account)
            .and_then(|vpn| {
                vpn.connect()
            });

    match vpn {
        Ok(_) => {

            let mut account_repository = AccountRepository::new("amakna.us")
                .map_err( |err| eprintln!("Impossible to initialize the account repository"))
                .unwrap();

            let result = account_repository.load_accounts();

/*
            let engine_session = engine::EngineSession::new(account);

            //start the bot engine for amakna
            let run = engine_session.and_then(| session | {
                engine::amakna::AmaknaBotEngine::new(session).and_then(
                    | mut amakna_bot_engine | {
                        amakna_bot_engine.run()
                    })
            });
*/

            let run: Result<(), String> = Err(String::from("err"));
            match run {
                Ok(_) => {
                    println!("Engine for botting has been successfully ran");
                },
                Err(err) => {
                    eprintln!("Engine has encountered errors during run, reason : {}", err);
                }
            }
        },
        Err(err) => eprintln!("Impossible to connect to the VPN")
    }

    println!("Leaving the program.");
}
