use chrono::{DateTime, NaiveDate, NaiveDateTime};
use chrono::Utc;
use rusoto_dynamodb::{DynamoDbClient, KeysAndAttributes, DynamoDb, BatchGetItemInput, AttributeValue, BatchGetItemError};
use std::collections::HashMap;
use env_logger;
pub use rusoto_credential::AwsCredentials;
pub use rusoto_credential::ProvideAwsCredentials;
use futures::future::Future;
use rusoto_core::Region;
use rusoto_credential::DefaultCredentialsProvider;
use std::str::from_utf8;
use std::error::Error;

static TABLE_ACCOUNTS_NAME: &'static str = "accounts";
static TABLE_ACCOUNTS_PRIMARY_KEY: &'static str = "domain";
static DATA_ATTRIBUTE_KEY: &'static str = "data";

///
/// represent an account for which to vote
///
pub struct Account {
    nickname: String,
    password: String,
    voted_at: Option<DateTime<Utc>>,
    disabled: bool,
}

///
/// list of available accounts
pub struct AccountRepository {
    accounts: Vec<Account>,
    dynamodb_client: DynamoDbClient,
    //represent the website of which the repository is belongs to
    domain: String,
}

impl Account {
    ///
    /// create a new account, by default the account is enabled
    pub fn new(nickname: String, password: String) -> Self {
        Account {
            nickname,
            password,
            voted_at: None,
            disabled: false
        }
    }

    ///
    /// set the vote attribute to now
    pub fn voted_now(&mut self) {
        self.voted_at = Some(Utc::now())
    }

    ///
    /// disable the account
    pub fn disable(&mut self) {
        self.disabled = true
    }

    ///
    /// check if the account can vote
    pub fn can_vote() {
        unimplemented!()
    }
}

///
/// different errors that could happens in the accountrepository context
pub enum AccountRepositoryError {
    FileNotExist,
    CannotLoadFile,
    GenericError,
    ErrorBatchItems,
    NoMatchingItem,
}

impl AccountRepository {

    ///
    /// create a new repository of accounts
    pub fn new(domain: &str) -> Self {
        let _ = env_logger::try_init();

        println!("create a new accountrepository");
        println!("Actual domain name is : {}", domain);
        Self {
            accounts: vec![],
            dynamodb_client: DynamoDbClient::new(Region::EuWest1),
            domain: String::from(domain),
        }
    }

    ///
    /// load accounts from dynamodb
    pub fn load_accounts(&mut self) -> Result<Vec<Account>, AccountRepositoryError> {

        println!("Load accounts from dynamodb");

        let default_cred_provider =
            DefaultCredentialsProvider::new().map_err(| err| {
                AccountRepositoryError::GenericError
            })?;

        let cred_future = default_cred_provider.credentials();

        let cred_res = cred_future.then(| fetched_cred | {

            match fetched_cred {
                Ok(ref cred) => {
                    println!("Actual default credentials : ");
                    println!("ACCESS TOKEN : {}", cred.aws_access_key_id());
                    println!("SECRET TOKEN : {}", cred.aws_secret_access_key());
                },
                Err(_) => {
                    println!("Impossible to print aws credentials");
                }
            }
            fetched_cred
        }).wait();

        match cred_res {
            Ok(cred) => {
                println!("Actual default credentials : ");
                println!("ACCESS TOKEN : {}", cred.aws_access_key_id());
                println!("SECRET TOKEN : {}", cred.aws_secret_access_key());
            },
            Err(_) => {
                eprintln!("impossible to get credentials");
            }
        }




        let mut account_items_pk_attr: HashMap<String, AttributeValue> = HashMap::new();
        account_items_pk_attr.insert(TABLE_ACCOUNTS_PRIMARY_KEY.into(), AttributeValue {
            b: None,
            bool: None,
            bs: None,
            l: None,
            m: None,
            n: None,
            ns: None,
            null: None,
            s: Some(self.domain.clone()),
            ss: None
        });

        let mut accounts_from_domain_input: HashMap<String, KeysAndAttributes> = HashMap::new();
        accounts_from_domain_input
            .insert(TABLE_ACCOUNTS_NAME.into(),
                    KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![account_items_pk_attr],
                        projection_expression: None,
                    });

        println!("BatchGetItemInput payload : {:?}", accounts_from_domain_input);

        let get_item_input = BatchGetItemInput {
            request_items: accounts_from_domain_input,
            return_consumed_capacity: None
        };


        let accounts =
            self.dynamodb_client.batch_get_item(get_item_input).sync();

        println!("foobar1");

        match accounts {
            Ok(batch_items) => {
                if let Some(resps) = batch_items.responses {
                    resps.get(self.domain.as_str())
                        .iter()
                        .for_each(| item | {
                            item.iter().for_each( | item_attributes | {
                                for (item_attrib_key, item_attrib_val) in item_attributes {
                                    println!("item attribute key {:?} || val {:?}",
                                             item_attrib_key, item_attrib_val);
                                }
                            })
                    });
                    return Ok(vec![
                        Account::new("toto".to_string(), "mdr".to_string())
                    ]);
                } else {
                    eprintln!("no items fetched");
                    Err(AccountRepositoryError::NoMatchingItem)
                }
            },
            Err(batch_err) => {
                eprintln!("Error during batch get item: {:?}", batch_err.description());
                match batch_err {
                    BatchGetItemError::Unknown(buffed_http_resp) => {
                        println!("Body err = {}", from_utf8(&buffed_http_resp.body).unwrap());
                    }
                    _ => {}
                }
                Err(AccountRepositoryError::GenericError)
            }
        }
    }
}