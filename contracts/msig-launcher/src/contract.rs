#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, SubMsgResult, WasmMsg,
};
use cw_utils::Duration;
use dao_interface::state::Admin::CoreModule;
use dao_interface::state::ModuleInstantiateInfo;
use dao_voting::pre_propose::PreProposeInfo;
use dao_voting::threshold::{PercentageThreshold, Threshold};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{MSigBuilder, MSIG, PENDING_MSIG};
use crate::{CW4_CODE_ID, MAIN_CODE_ID, PRE_PROPOSE_CODE_ID, PROPOSAL_CODE_ID, VOTING_CODE_ID};
/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:msig-launcher";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Instantiate {
            label,
            name,
            description,
            image_url,
            max_voting_period,
            members,
        } => {
            let msg = dao_interface::msg::InstantiateMsg {
                admin: None,
                name,
                description,
                image_url,
                automatically_add_cw20s: false,
                automatically_add_cw721s: false,
                voting_module_instantiate_info: ModuleInstantiateInfo {
                    code_id: VOTING_CODE_ID,
                    msg: to_json_binary(&cw4_voting::msg::InstantiateMsg {
                        cw4_group_code_id: CW4_CODE_ID,
                        initial_members: members,
                    })?,
                    admin: Some(CoreModule {}),
                    funds: vec![],
                    label: format!("{}-voting-module", label),
                },
                proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
                    code_id: PROPOSAL_CODE_ID,
                    msg: to_json_binary(&dao_proposal_single::msg::InstantiateMsg {
                        threshold: Threshold::ThresholdQuorum {
                            threshold: PercentageThreshold::Majority {},
                            quorum: PercentageThreshold::Majority {},
                        },
                        max_voting_period: Duration::Time(max_voting_period),
                        min_voting_period: None,
                        only_members_execute: true,
                        allow_revoting: false,
                        pre_propose_info: PreProposeInfo::ModuleMayPropose {
                            info: ModuleInstantiateInfo {
                                code_id: PRE_PROPOSE_CODE_ID,
                                msg: to_json_binary(&dao_pre_propose_base::msg::InstantiateMsg {
                                    deposit_info: None,
                                    open_proposal_submission: false,
                                    extension: Empty {},
                                })?,
                                admin: Some(CoreModule {}),
                                funds: vec![],
                                label: format!("{}-pre-proposal-module", label),
                            },
                        },
                        close_proposal_on_execution_failure: false,
                        veto: None,
                    })?,
                    admin: Some(CoreModule {}),
                    funds: vec![],
                    label: format!("{}-proposal-module", label),
                }],
                initial_items: None,
                dao_uri: None,
            };

            if PENDING_MSIG.exists(deps.storage) {
                return Err(ContractError::UnexpectedDoubleTx {});
            }

            PENDING_MSIG.save(deps.storage, &(label.clone(), info.sender))?;

            Ok(Response::default().add_submessage(SubMsg {
                id: 0,
                msg: WasmMsg::Instantiate {
                    admin: None,
                    code_id: MAIN_CODE_ID,
                    msg: to_json_binary(&msg)?,
                    funds: vec![],
                    label,
                }
                .into(),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            }))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::MSig { label } => to_json_binary(&MSIG.load(deps.storage, label)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut resp = Response::default();
    let (label, sender) = PENDING_MSIG.load(deps.storage)?;
    PENDING_MSIG.remove(deps.storage);

    let mut builder = MSigBuilder::new(sender);

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
                        builder.set_contract(code_id.parse::<u64>().unwrap(), address)?;
                    }
                }
            }
            Ok(())
        }
        SubMsgResult::Err(err) => Err(ContractError::ReplyError(err)),
    }?;

    let msig = builder.build()?;

    msig.append_attrs(&mut resp.attributes);

    MSIG.save(deps.storage, label, &msig)?;

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::MSig;
    use cosmwasm_std::Addr;
    use cw4::Member;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn instantiate() {
        let mut app = App::default();
        // Store all Dao contracts
        // Base
        app.store_code(Box::new(
            ContractWrapper::new(
                dao_dao_core::contract::execute,
                dao_dao_core::contract::instantiate,
                dao_dao_core::contract::query,
            )
            .with_reply(dao_dao_core::contract::reply),
        ));
        // Voting
        app.store_code(Box::new(
            ContractWrapper::new(
                cw4_voting::contract::execute,
                cw4_voting::contract::instantiate,
                cw4_voting::contract::query,
            )
            .with_reply(cw4_voting::contract::reply),
        ));
        // Proposal
        app.store_code(Box::new(
            ContractWrapper::new(
                dao_proposal_single::contract::execute,
                dao_proposal_single::contract::instantiate,
                dao_proposal_single::contract::query,
            )
            .with_reply(dao_proposal_single::contract::reply),
        ));
        // Pre Propose
        app.store_code(Box::new(ContractWrapper::new(
            dao_pre_propose_single::contract::execute,
            dao_pre_propose_single::contract::instantiate,
            dao_pre_propose_single::contract::query,
        )));
        // Cw4
        app.store_code(Box::new(ContractWrapper::new(
            cw4_group::contract::execute,
            cw4_group::contract::instantiate,
            cw4_group::contract::query,
        )));

        // Msig launcher
        let launcher_code_id = app.store_code(Box::new(
            ContractWrapper::new(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_reply(crate::contract::reply),
        ));

        let launcher = app
            .instantiate_contract(
                launcher_code_id,
                Addr::unchecked("admin"),
                &InstantiateMsg {},
                &[],
                "label",
                None,
            )
            .unwrap();

        let res = app
            .execute_contract(
                Addr::unchecked("user"),
                launcher.clone(),
                &ExecuteMsg::Instantiate {
                    label: "msig".to_string(),
                    name: "SomeWallet".to_string(),
                    description: "SomeDescription".to_string(),
                    image_url: Some("Image".to_string()),
                    max_voting_period: 100,
                    members: vec![
                        Member {
                            addr: "addr1".to_string(),
                            weight: 1,
                        },
                        Member {
                            addr: "addr2".to_string(),
                            weight: 1,
                        },
                        Member {
                            addr: "addr3".to_string(),
                            weight: 1,
                        },
                    ],
                },
                &[],
            )
            .unwrap();

        // Get the last event which should be this contract's reply
        let relevant_event = res.events.last().unwrap();

        assert_eq!(
            relevant_event.attributes,
            vec![
                ("_contract_address", "contract0"),
                ("creator", "user"),
                ("dao_dao_address", "contract1"),
                ("voting_address", "contract2"),
                ("proposal_address", "contract4"),
                ("pre_propose_address", "contract5"),
                ("cw4_address", "contract3"),
            ]
        );

        let res: MSig = app
            .wrap()
            .query_wasm_smart(
                launcher.clone(),
                &QueryMsg::MSig {
                    label: "msig".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            res,
            MSig {
                creator: Addr::unchecked("user"),
                dao_dao_contract: "contract1".to_string(),
                voting_contract: "contract2".to_string(),
                proposal_contract: "contract4".to_string(),
                pre_propose_contract: "contract5".to_string(),
                cw4_contract: "contract3".to_string(),
            }
        )
    }
}
