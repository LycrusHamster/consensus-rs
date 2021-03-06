use std::sync::Arc;
use std::str::FromStr;
use std::str::Utf8Error;

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use ethereum_types::H160;
use parking_lot::RwLock;

use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::EMPTY_HASH;

use crate::{
    types::{Timestamp, Gas, Difficulty, Height, EMPTY_ADDRESS},
    types::block::{Block, Header},
    types::votes::{decrypt_commit_bytes, encrypt_commit_bytes, Votes},
    types::{Validator, Validators},
    config::GenesisConfig,
    common,
};
use super::{
    ledger::Ledger,
};

pub(crate) fn store_genesis_block(genesis_config: &GenesisConfig, ledger: Arc<RwLock<Ledger>>) -> Result<(), String> {
    use chrono::{Local, DateTime, ParseError};
    let mut ledger = ledger.write();
    if let Some(genesis) =  ledger.get_genesis_block() {
        info!("Genesis hash:{:?}", genesis.hash());
        ledger.reload_meta();
        return Ok(());
    }
    // add validators
    {
        let validators: Validators = genesis_config.validator.iter().map(|validator| {
            common::string_to_address(validator).unwrap()
        }).map(|address| {
            Validator::new(address)
        }).collect();
        ledger.add_validators(validators);
    }

    // TODO Add more xin
    {
        let proposer = common::string_to_address(&genesis_config.proposer)?;
        let epoch_time: DateTime<Local> = {
            let epoch_time_str = genesis_config.epoch_time.to_string();
            DateTime::from_str(&epoch_time_str)
        }.map_err(|err: ParseError| err.to_string())?;

        let extra = genesis_config.extra.as_bytes().to_vec();
        let mut header = Header::new(EMPTY_HASH, proposer, EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                                     0, 0, 0, genesis_config.gas_used + 10, genesis_config.gas_used,
                                     epoch_time.timestamp() as Timestamp, None, Some(extra));
        let block = Block::new(header, vec![]);
        ledger.add_genesis_block(&block);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::random_dir;
    use cryptocurrency_kit::ethkey::{Generator, Random};
    use kvdb_rocksdb::Database;
    use crate::store::schema::Schema;
    use crate::core::ledger::{Ledger, LastMeta};
    use lru_time_cache::LruCache;

    #[test]
    fn t_genesis_block() {
        let secret = Random.generate().unwrap();

        let database = Database::open_default(&random_dir()).map_err(|err| err.to_string()).unwrap();
        let schema = Schema::new(Arc::new(database));
        let mut ledger = Ledger::new(
            LastMeta::new_zero(),
            LruCache::with_capacity(1 << 10),
            LruCache::with_capacity(1 << 10),
            vec![],
            schema,
        );

        let mut header = Header::new(EMPTY_HASH, Address::from(10), EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                                     0, 0, 0, 10, 10,
                                     192, None, Some(vec![12, 1]));
        let block = Block::new(header, vec![]);

        ledger.add_genesis_block(&block);

        assert_eq!(false, ledger.get_block_hash_by_height(0).is_none());
        assert_eq!(true, ledger.get_block_hash_by_height(1).is_none());
    }

    #[test]
    fn t_back_block() {
        let secret = Random.generate().unwrap();

        let database = Database::open_default(&random_dir()).map_err(|err| err.to_string()).unwrap();
        let schema = Schema::new(Arc::new(database));
        let mut ledger = Ledger::new(
            LastMeta::new_zero(),
            LruCache::with_capacity(1 << 10),
            LruCache::with_capacity(1 << 10),
            vec![],
            schema,
        );

        let mut header = Header::new(EMPTY_HASH, Address::from(10), EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                                     0, 0, 0, 10, 10,
                                     192, None, Some(vec![12, 1]));
        let block = Block::new(header, vec![]);

        ledger.add_genesis_block(&block);
        ledger.reload_meta();

        (1_u64..10).for_each(|height|{
            let mut header = Header::new(EMPTY_HASH, Address::from(10), EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                                         0, 0, height, 10, 10,
                                         192, None, Some(vec![12, 1]));
            let block = Block::new(header, vec![]);

            ledger.add_block(&block);
        });

        (1_u64..10).for_each(|height|{
            let block = ledger.get_block_by_height(height).unwrap();
            let block1 = ledger.get_block(&block.hash()).unwrap();
            println!("{:?}", block);
            println!("|{:?}", block1);

        });

//        let schema = ledger.get_schema();
//        for block in schema.blocks().iter() {
//            println!("{:?}", block);
//        }
//
//        println!("last_block {:?}", ledger.get_last_block());
    }

    #[test]
    fn t_exists_db() {
//        let database = Database::open_default("/tmp/block/c1").map_err(|err| err.to_string()).unwrap();
//        let schema = Schema::new(Arc::new(database));
//        for key in schema.blocks().keys() {
//            println!("{:?}", key);
//        }

//        for block in schema.blocks().iter() {
//            println!("{:?}", block);
//        }
        
//        for value in schema.blocks().values() {
//            println!("{:?}", value);
//        }
    }
}