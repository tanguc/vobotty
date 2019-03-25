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
use rusoto_core::HttpClient;
use rusoto_credential::ProfileProvider;
use std::env;

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
    accounts: Box<Vec<Account>>,
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
    CannotLoadSpecificCredentialsProfile,
    CannotCreateHttpClient,
}

impl AccountRepository {

    ///
    /// create a new repository of accounts
    pub fn new(domain: &str) -> Result<Self, AccountRepositoryError> {
        let _ = env_logger::try_init();

        println!("create a new accountrepository");
        println!("Actual domain name is : {}", domain);
        let request_dispatcher = HttpClient::new()
            .map_err(|err| AccountRepositoryError::CannotCreateHttpClient)?;
        let aws_credentials_provider = ProfileProvider::new()
            .map_err(| err | AccountRepositoryError::CannotLoadSpecificCredentialsProfile)?;

        Ok(Self {
            accounts: Box::new(Vec::new()),
            dynamodb_client: DynamoDbClient::new_with(
                request_dispatcher,
                aws_credentials_provider,
                Region::EuWest1),
            domain: String::from(domain),
        })
    }

    ///
    /// load accounts from dynamodb
    pub fn load_accounts(&mut self) -> Result<Vec<Account>, AccountRepositoryError> {

        println!("Load batch_result_accounts from dynamodb");

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

        let mut accounts_from_domain: HashMap<String, KeysAndAttributes> = HashMap::new();
        accounts_from_domain
            .insert(TABLE_ACCOUNTS_NAME.into(),
                    KeysAndAttributes {
                        attributes_to_get: None,
                        consistent_read: None,
                        expression_attribute_names: None,
                        keys: vec![account_items_pk_attr],
                        projection_expression: None,
                    });

        println!("BatchGetItemInput payload : {:?}", accounts_from_domain);

        let batch_domain_accounts = BatchGetItemInput {
            request_items: accounts_from_domain,
            return_consumed_capacity: None
        };


        let batch_result_accounts =
            self.dynamodb_client
                .batch_get_item(batch_domain_accounts)
                .sync()
                .map_err(|err| AccountRepositoryError::GenericError)?;

        println!("foobar1");

        let mut accounts: Vec<Account> = Vec::new();

        if let Some(accounts_items) = batch_result_accounts.responses {
            if let accounts_items = accounts_items.get(TABLE_ACCOUNTS_PRIMARY_KEY) {
                println!("Actuals accounts items : {:?}", accounts_items);
            } else {
                eprintln!("There aren't any data for the ")
            }
        } else {
            eprintln!("There aren't any accounts in the table for this domain name");
        }

        Ok(accounts)
    }
}