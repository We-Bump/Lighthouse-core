use cosmwasm_std::{DepsMut, Reply, Response};
use cw_utils::parse_reply_instantiate_data;

use crate::{state::Lighthouse, ContractError};

impl<'a> Lighthouse<'a> {
    pub fn reply(&self, deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
        let instantiate_info = self.instantiates.load(deps.storage, msg.id)?;

        let reply = parse_reply_instantiate_data(msg.clone()).unwrap();

        let mut collection = instantiate_info.clone();
        collection.collection_address = Some(reply.contract_address);

        if self.collections.has(deps.storage, collection.collection_address.clone().unwrap()) {
            return Err(ContractError::CollectionExists {});
        }

        self.collections.save(deps.storage, collection.collection_address.clone().unwrap(), &collection)?;
        self.instantiates.remove(deps.storage, msg.id);


        Ok(
            Response::new()
                .add_attribute("register_collection", "success")
                .add_attribute("collection", collection.collection_address.unwrap())
                .add_attribute("type", collection.collection_type)
                .add_attribute("chain", collection.chain)
        )
    }
}