use crate::state::{MSigBuilder, MSIG, MSIG_CODE_IDS, PENDING_MSIG};
use crate::ContractError;
use archway_proto::archway::rewards::v1::{ContractMetadata, MsgSetContractMetadata};
use archway_proto::prost::{Message, Name};
use cosmwasm_std::{
    entry_point, Binary, CosmosMsg, DepsMut, Env, Reply, Response, SubMsg, SubMsgResult, WasmMsg,
};

fn set_metadatas(resp: &mut Response, env: &Env, dao_core: String, target_contracts: &[&str]) {
    for contract in target_contracts {
        // Set the contracts metadata to automatically withdraw into the dao
        let msg = MsgSetContractMetadata {
            sender_address: env.contract.address.to_string(),
            metadata: Some(ContractMetadata {
                contract_address: contract.to_string(),
                owner_address: env.contract.address.to_string(),
                rewards_address: dao_core.clone(),
                withdraw_to_wallet: true,
            }),
        };
        resp.messages.push(SubMsg::new(CosmosMsg::Stargate {
            type_url: MsgSetContractMetadata::type_url(),
            value: Binary::from(msg.encode_to_vec()),
        }));

        // Change the contract admin to dao core
        resp.messages.push(SubMsg::new(WasmMsg::UpdateAdmin {
            contract_addr: contract.to_string(),
            admin: dao_core.clone(),
        }));
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(dead_code)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut resp = Response::default();
    let code_ids = MSIG_CODE_IDS.load(deps.storage)?;
    let (sender, block) = PENDING_MSIG.load(deps.storage)?;
    PENDING_MSIG.remove(deps.storage);

    let mut builder = MSigBuilder::default();

    match msg.result {
        SubMsgResult::Ok(result) => {
            for event in result.events {
                // Look for instantiate type
                if event.ty == "instantiate" {
                    // Look for both addr and code id
                    let mut address = None;
                    let mut code_id = None;
                    for attr in event.attributes {
                        if attr.key == "_contract_address" {
                            address = Some(attr.value);
                        } else if attr.key == "code_id" {
                            code_id = Some(attr.value);
                        }
                    }

                    // If both are found, match them into their appropriate address
                    if let Some((address, code_id)) = address.zip(code_id) {
                        builder.set_contract(
                            &code_ids,
                            code_id.parse::<u64>().unwrap(),
                            address,
                        )?;
                    }
                }
            }
            Ok(())
        }
        SubMsgResult::Err(err) => Err(ContractError::ReplyError(err)),
    }?;

    let msig = builder.build()?;

    msig.append_attrs(&sender, &mut resp.attributes);

    MSIG.save(deps.storage, (sender, block), &msig)?;

    // Set the contract metadata
    set_metadatas(
        &mut resp,
        &env,
        msig.dao_dao_contract.clone(),
        &[
            &msig.dao_dao_contract,
            &msig.voting_contract,
            &msig.proposal_contract,
            &msig.pre_propose_contract,
        ],
    );

    Ok(resp)
}
