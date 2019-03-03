pub mod engine;
mod vpn;


use crate::engine::
use ::engine;


fn main() {
    let account = engine::Account::new(
        "morter".to_string(), "s45LrPVqijS1".to_string());

    let vpn_account = vpn::VpnCredentials::new(
        "sergen.tanguc@gmail.com".to_string(),
        "oizjdofi".to_string());

    let vpn =
        vpn::Vpn::new(vpn_account)
            .and_then(|vpn| {
                vpn.connect()
            });

    match vpn {
        Ok(_) => {
            let engine_session = engine::EngineSession::new(account);

            //start the bot engine for amakna
            let run = engine_session.and_then(| session | {
                engine::amakna::AmaknaBotEngine::new(session).and_then(
                    |amakna_bot_engine | {
                        amakna_bot_engine.run()
                    })
            });

            if run.is_ok() {
                println!("Engine for botting has been successfully ran");
            } else {
                eprintln!("Engine has encountered errors during run");
            }

        },
        Err(err) => eprintln!("Impossible to connect to the VPN")
    }

    println!("Leaving the program.");
}
