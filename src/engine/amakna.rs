/// Amakna module which represents the implementations
/// for the bot engine
///
use reqwest;
use reqwest::Url;
use super::*;

pub struct AmaknaBotEngine {
    session: EngineSession,
}

impl EngineExecutor for AmaknaBotEngine {
    fn run(&self) -> Result<bool, String> {
        println!("Start to run the Amakna Bot Engine");

        self.login();

        Ok(false)
    }

    ///static content specific to amakna
    fn get_website() -> Website {
        Website {
            name: "Amakna".to_string(),
            host: "https://ganymede.ws".to_string(),
            login_endpoint_path: "".to_string(),
            vote_endpoint_path: "".to_string()
        }
    }
}

impl AmaknaBotEngine {
    pub fn new(session: EngineSession) -> Result<Self, String> {
        Ok(AmaknaBotEngine {
            session
        })
    }

    ///
    /// Try to login to the website with the given payload
    ///
    fn login(&self) -> Result<bool, String> {

        let mut success = false;

        println!("Trying use following payload;");
        println!("{:?}", self.session.get_account());

        let mut url = Url::parse(Self::get_website().host.as_ref())
            .and_then(| url | {
                url.join(Self::get_website().login_endpoint_path.as_ref())
            })
            .map_err(| err | {
                err.to_string(
            )
        });

        match url {
            Ok(url) => {
                let req_builder = self.session.get_http_client()
                    .post(url);

                //try request synchronously
                let result = req_builder.send();

                match result {
                    Ok(mut resp) => {
                        println!("Response is OK");

                        println!("Response text : {:?}", resp.text().unwrap());

                        if resp.status().is_success() {
                            println!("Its a success");
                        } else {
                            eprintln!("Its a en error as status");
                        }

                        println!("headers : {:?}", resp.headers());
                        Ok(true)
                    },
                    Err(err) => {
                        eprintln!("Response has failed");
                        eprintln!("Response error : {:?}", err.to_string());
                        Err("Response failed".to_string())
                    },
                }
            },
            Err(err) => {
                eprintln!("Error during run : {}", err);
                Err(err)
            }
        }
    }
}