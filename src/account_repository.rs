use chrono::{DateTime, NaiveDate, NaiveDateTime};
use chrono::Utc;
use rusoto_dynamodb::{DynamoDbClient, KeysAndAttributes, DynamoDb, BatchGetItemInput, AttributeValue, BatchGetItemError};
use std::collections::HashMap;
use env_logger;
use futures::future::Future;
use rusoto_core::{Region, ProvideAwsCredentials, CredentialsError};
use std::str::from_utf8;
use std::error::Error;
use rusoto_core::HttpClient;
use std::env;
use rusoto_core::DefaultCredentialsProvider;
use rusoto_core::credential::AwsCredentials;

static TABLE_ACCOUNTS_NAME: &'static str = "accounts";
static TABLE_ACCOUNTS_PRIMARY_KEY: &'static str = "domain";
static DATA_ATTRIBUTE_KEY: &'static str = "data";

/// Account belongs to a domain
#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    nickname: String,
    password: String,
    voted_at: Option<DateTime<Utc>>,
    disabled: bool,
}

/// list of available accounts
pub struct AccountRepository {
    accounts: Box<Vec<Account>>,
    dynamodb_client: Box<DynamoDbClient>,
    //represent the website of which the repository is belongs to
    domain: String,
}

impl Account {
    /// create a new account, by default the account is enabled
    pub fn new(nickname: String, password: String) -> Self {
        Account {
            nickname,
            password,
            voted_at: None,
            disabled: false,
        }
    }

    /// set the vote attribute to now
    pub fn voted_now(&mut self) {
        self.voted_at = Some(Utc::now())
    }

    /// disable the account
    pub fn disable(&mut self) {
        self.disabled = true
    }

    /// check if the account can vote
    pub fn can_vote() {
        unimplemented!()
    }
}

/// Errors
pub enum AccountRepositoryError {
    FileNotExist,
    CannotLoadFile,
    GenericError,
    ErrorBatchItems,
    NoMatchingItem,
    CannotLoadSpecificCredentialsProfile,
    CannotCreateHttpClient,
    ErrorDuringBatchGetItemRequest,
    EmptyResponseForDomain,
    CouldNotGetCredsFromEnvironment,
    CannotParseAccountsFromJson,
    NoData,
}

impl AccountRepository {
    /// Create a new repository(DAO) of accounts
    /// If credentials are set in the env, it tries to login with them, otherwise it uses
    /// the .credentials file
    pub fn new(domain: &str) -> Result<Self, AccountRepositoryError> {
        let _ = env_logger::try_init();

        println!("create a new accountrepository");
        println!("Actual domain name is : {}", domain);
        let request_dispatcher = HttpClient::new()
            .map_err(|err| AccountRepositoryError::CannotCreateHttpClient)?;

        //if credentials are set in the env we use them first
        let dynamodb_client =
            if env::var("AWS_ACCESS_KEY_ID").is_ok() &&
                env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
                Box::new(DynamoDbClient::new_with(
                    request_dispatcher,
                    rusoto_core::credential::EnvironmentProvider::default(),
                    Region::EuWest1))
            } else {
                Box::new(DynamoDbClient::new(Region::EuWest1))
            };

        Ok(Self {
            accounts: Box::new(Vec::new()),
            dynamodb_client,
            domain: String::from(domain),
        })
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
            let mut accounts_data = Err(AccountRepositoryError::NoData);
            domain_accounts.get(TABLE_ACCOUNTS_NAME)
                .iter()
                .flat_map(|arg| arg.iter())
                .filter(|domain_item_attribute| {
                    domain_item_attribute.contains_key(DATA_ATTRIBUTE_KEY)
                })
                .for_each(|domain_item_data_attribute| {
                    println!("Actual DATA payload : {:?}",
                             domain_item_data_attribute.get(DATA_ATTRIBUTE_KEY));
                    let domain_item_data_attribute = domain_item_data_attribute
                        .get(DATA_ATTRIBUTE_KEY);
                    match domain_item_data_attribute {
                        Some(data_value) => {
                            println!("JSON payload : {:?}", data_value.s);
                            accounts_data = data_value.s.clone()
                                .ok_or_else(|| AccountRepositoryError::NoData);
                        },
                        None => {
                            eprintln!("Cannot retrieve data value");
                        }
                    }
                });
            accounts_data
        } else {
            eprintln!("Dynamodb returned empty collections for accounts table");
            Err(AccountRepositoryError::EmptyResponseForDomain)
        }
    }

    /// Parse JSON payload
    /// Return a list of accounts
    fn parse(&self, payload: &str) -> Result<Box<Vec<Account>>, AccountRepositoryError> {
        println!("Trying to parse JSON payload");


        let accounts: Vec<Account> = serde_json::from_str(payload)
            .map_err(|err| {
                eprintln!("Got error during deserialization of JSON which contains account\
                , reason => {:?}", err);
                AccountRepositoryError::CannotParseAccountsFromJson
            })?;
        println!("Parsed Vec accounts = {:?}", accounts);

        Ok(Box::new(accounts))
    }

    /// load accounts from dynamodb
    pub fn load_accounts(&mut self) -> Result<Box<Vec<Account>>, AccountRepositoryError> {
        let mut accounts: Vec<Account> = Vec::new();
        println!("Load accounts from dynamodb");

        let json_accounts = self.retrieve_data()?;
        self.parse(json_accounts.as_str())
    }
}