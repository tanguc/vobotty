/// Amakna module which represents the implementations
/// for the bot engine
///
use reqwest;
use reqwest::Url;
use super::{
    EngineSession,
    EngineExecutor,
    Website,
};
use regex;
use reqwest::Response;
use hyper::HeaderMap;
use http::header::HeaderValue;
use std::collections::HashMap;

pub struct AmaknaBotEngine {
    session: EngineSession,
    /// headerMap to keep cookies
    cookies: Box<reqwest::header::HeaderMap>,
    nickname: NicknameState,
}

impl EngineExecutor for AmaknaBotEngine {
    fn run(&mut self) -> Result<bool, String> {
        println!("Start to run the Amakna Bot Engine");


        //try to login
        self.login()?;

        //try to check if the login has been successful
        println!("Trying to verify if the client is logged.");
        self.check_connected()?;


        self.vote()?;

        Ok(true)
/*
        self.vote()?;
*/
    }

    ///static content specific to amakna
    fn get_website() -> Website {
        Website {
            name: "Amakna".to_string(),
            host: "https://ganymede.ws".to_string(),
            login_endpoint_path: "connected".to_string(),
            vote_endpoint_path: "vote".to_string(),
            index_endpoint_path: "index".to_string(),
        }
    }
}

enum NicknameState {
    NotFoundYet,
    Found(String),
}

impl AmaknaBotEngine {
    pub fn new(session: EngineSession) -> Result<Self, String> {
        Ok(AmaknaBotEngine {
            session,
            cookies: Box::new(HeaderMap::new()),
            nickname: NicknameState::NotFoundYet,
        })
    }

    ///
    /// Check if the account has been connected
    /// Return the pseudo of the current account displayed in the website
    fn check_connected(&mut self) -> Result<bool, String> {
        println!("Checking the client has been connected.");

        let url = self.build_index_url()?;

        let mut resp = self.session.http_client.get(url)
            .headers((*self.cookies).clone())
            .send()
            .map_err(| err | {
                eprintln!("Error happened during the GET in login page to check if the \
                client has been connected");
                String::from("Error happened during the GET in login page to check if the \
                client has been connected")
            })?;

        let resp_body = resp.text().map_err(|err| {
            String::from("Impossible to get the body of the response.")
        })?;

        println!("Response body in check phase: {}", resp_body);

        let pattern_member_div = "<div id=\"member\">";

        if let Some(index) = resp_body.find(&pattern_member_div) {
            println!("Found the index of the pattern : {}", index);
            Ok(true)
/*
            let regex = regex::Regex::new(pattern_member_div)
                .map_err(| err | {
                    String::from("Cannot create the regex pattern object.")
                })?;

            let nickname_captures = regex.captures(&resp_body)
                .ok_or(String::from("Cannot captures nickname groups from response"))?;

            if nickname_captures.len() > 0 {
                let nickname = nickname_captures
                    .get(1)
                    .ok_or(String::from(
                        "Cannot find any account nickname in the response"))?;
                println!("The nickname of the account is : {}", nickname.as_str());
                self.nickname = NicknameState::FOUND(String::from(nickname.as_str()));
                Ok(true)
            } else {
                Err(String::from("Impossible to find out the nickname of the account"))
            }
*/
        } else {
            Err(String::from("No clue in the response body if the client has been connected."))
        }
    }

    fn build_url_from_endpoint(&self, endpoint: &String) -> Result<Url, String> {

        Ok(Url::parse(Self::get_website().host.as_ref())
            .and_then(| url | {
                url.join(endpoint.as_ref())
            })
            .map_err(| err | {
                String::from("Impossible to build le login url")
            })?)

    }

    ///Build the login url
    fn build_login_url(&self) -> Result<Url, String> {
        self.build_url_from_endpoint(&Self::get_website().login_endpoint_path)
    }


    ///Build the index url
    fn build_index_url(&self) -> Result<Url, String> {
        self.build_url_from_endpoint(&Self::get_website().index_endpoint_path)
    }

    ///Build the vote url
    fn build_vote_url(&self) -> Result<Url, String> {
        self.build_url_from_endpoint(&Self::get_website().vote_endpoint_path)
    }

    ///
    /// Try to login to the website with the given payload
    ///
    fn login(&mut self) -> Result<bool, String> {

        let mut success = false;

        println!("Trying use following payload;");
        println!("{:?}", self.session.get_account());

        let mut url = self.build_login_url()?;

        println!("Login url built = [{}]", url);
        let mut login_form_data =
            self.session.account.get_param_map();
        login_form_data.insert(String::from("login"),
                               String::from("Connexion"));

        let req_builder = self.session.get_http_client()
            .post(url)
            .form(&login_form_data);

        //try request synchronously
        let mut resp = req_builder.send().map_err(| err | {
            String::from("Impossible to send the request.")
        })?;

        println!("Response is OK");
        println!("Response status : {}", resp.status().as_str());
        println!("Response text : {:?}", resp.text().unwrap());

        let status_code = (resp.status().as_u16() / 100) as u32;
        if status_code != 4 || status_code != 5 {
            println!("Its a success");
            //save cookies for future purpose (keep connection alive)
            self.save_cookies(resp.headers());
        } else {
            eprintln!("Its a en error as status");
            return Err(String::from("Error during login, error 4XX returned."))
        }

        println!("headers : {:?}", resp.headers());
        println!("Response code : {}", resp.status());
        Ok(true)
    }

    ///
    /// Start to vote
    /// The client must be connected before to try this action
    ///
    fn vote(&self) -> Result<bool, String> {
        println!("Trying to vote.");

        let mut url = self.build_vote_url()?;
        url.set_query(Some("true=rpg"));

        let resp = self.session.http_client
            .post(url)
            .headers((*self.cookies.clone()))
            .send()
            .map_err(| err | {
                err.to_string()
            })?;

        if resp.status().is_success() {
            println!("Vote returned a success status");
            Ok(true)
        } else {
            eprintln!("Vote request has returned an error status: {}", resp.status().to_string());
            Err(resp.status().to_string())
        }
    }


    /// Save the cookies to be able to keep connection alive
    fn save_cookies(&mut self, headers: &HeaderMap) -> bool {
        println!("Trying to save cookies.");

        for (key, val) in headers.iter() {
            println!("Trying to save cookie key [{:?}] & val [{:?}]", key, val);
        }

        let set_cookies_headers = headers.iter()
            .filter(| header_item | {
                let (header_key, _) = header_item;
                header_key.as_str().eq("set-cookie")
            })
            .for_each(| set_cookie_header | {
                let (_, set_cookie_header_val) = set_cookie_header;

                if let Ok(set_cookie_parsed) = set_cookie_header_val.to_str() {
                    if let Some(cookie_val) = set_cookie_parsed.split(';').next() {
                        println!("Trying to save raw cookie val: {}", cookie_val);
                        if let Ok(cookie_val) = HeaderValue::from_str(cookie_val) {
                            self.cookies.append(reqwest::header::COOKIE,  cookie_val);
                        } else {
                            eprintln!("Error happened during the insertion\
                             of the returned cookie from the server");
                        }
                    }
                }
            });

        for (key, val) in self.cookies.iter() {
            println!("After saving :");
            println!("Trying to save cookie key [{:?}] & val [{:?}]", key, val);
        }

        true
    }
}