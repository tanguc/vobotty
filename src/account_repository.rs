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
            disabled: false,
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
    ErrorDuringBatchGetItemRequest,
    EmptyResponseForDomain,
}

impl AccountRepository {
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

    /// Print credentials for debug purposes
    fn print_credentials_debug() {
        let default_cred_provider =
            DefaultCredentialsProvider::new().map_err(|err| {
                eprintln!("Cannot create default credentials provider");
                ()
            });
        match default_cred_provider {
            Ok(default_cred_provider ) => {
                let cred_future = default_cred_provider.credentials();

                let cred_res = cred_future.then(|fetched_cred| {
                    match fetched_cred {
                        Ok(ref cred) => {
                            println!("Actual default credentials : ");
                            println!("ACCESS TOKEN : {}", cred.aws_access_key_id());
                            println!("SECRET TOKEN : {}", cred.aws_secret_access_key());
                        }
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
                    }
                    Err(_) => {
                        eprintln!("impossible to get credentials");
                    }
                }
            },
            Err(_) => { }
        }
    }

    /// Retrieve all accounts data related to the domain
    /// The attribute name of the item is DATA_ATTRIBUTE_KEY
    /// Returns the payload in JSON format
    fn retrieve_data(&self) -> Result<String, AccountRepositoryError> {
        let mut input_retrieve_domain_key: HashMap<String, AttributeValue> = HashMap::new();
        input_retrieve_domain_key.insert(TABLE_ACCOUNTS_PRIMARY_KEY.into(), AttributeValue {
            b: None,
            bool: None,
            bs: None,
            l: None,
            m: None,
            n: None,
            ns: None,
            null: None,
            s: Some(self.domain.clone()),
            ss: None,
        });

        let mut batch_retrieve_domain_from_table: HashMap<String, KeysAndAttributes> = HashMap::new();
        batch_retrieve_domain_from_table
            .insert(TABLE_ACCOUNTS_NAME.into(),
                    KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![input_retrieve_domain_key],
                        projection_expression: None,
                    });

        println!("BatchGetItemInput payload : {:?}", batch_retrieve_domain_from_table);

        let get_item_input = BatchGetItemInput {
            request_items: batch_retrieve_domain_from_table,
            return_consumed_capacity: None,
        };


        let batch_accounts = self.dynamodb_client
            .batch_get_item(get_item_input)
            .sync()
            .map_err(|err| AccountRepositoryError::ErrorDuringBatchGetItemRequest)?;

        println!("God responses :o");
        println!("Raw: {:?}", batch_accounts);

        if let Some(domain_accounts) = batch_accounts.responses {
            domain_accounts.get(TABLE_ACCOUNTS_NAME)
                .iter()
                .for_each(|domain_item| {
                    domain_item.iter().filter(|domain_item_attribute| {
                        domain_item_attribute.contains_key(DATA_ATTRIBUTE_KEY)
                    }).for_each(|domain_item_data_attribute| {
                        println!("Actual DATA payload : {:?}",
                                 domain_item_data_attribute.get(DATA_ATTRIBUTE_KEY));
                        let domain_item_data_attribute = domain_item_data_attribute
                            .get(DATA_ATTRIBUTE_KEY);
                        match domain_item_data_attribute {
                            Some(data_value) => {
                                println!("JSON payload : {:?}", data_value.s.);
                            },
                            None => {
                                eprintln!("Cannot retrieve data value");
                            }
                        }
                    })
                });
            Ok(String::from("toto"))
        } else {
            eprintln!("Dynamodb returned empty collections for accounts table");
            Err(AccountRepositoryError::EmptyResponseForDomain)
        }
    }


    ///
    /// load accounts from dynamodb
    pub fn load_accounts(&mut self) -> Result<Vec<Account>, AccountRepositoryError> {
        let mut accounts: Vec<Account> = Vec::new();
        println!("Load accounts from dynamodb");

        self.retrieve_data();

        Ok(accounts)
    }
}