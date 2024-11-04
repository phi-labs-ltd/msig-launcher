use crate::state::{MSIG_CODE_IDS, PENDING_MSIG};
use crate::ContractError;
use cosmwasm_std::{to_json_binary, DepsMut, Empty, Env, MessageInfo, Response, SubMsg, WasmMsg};
use cw_utils::Duration;
use dao_interface::state::Admin::Address;
use dao_interface::state::ModuleInstantiateInfo;
use dao_voting::pre_propose::PreProposeInfo;
use dao_voting::threshold::{PercentageThreshold, Threshold};

#[allow(clippy::too_many_arguments)]
pub fn execute_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    image_url: Option<String>,
    min_voting_period: u64,
    max_voting_period: u64,
    members: Vec<cw4::Member>,
) -> Result<Response, ContractError> {
    let label = format!("{}-{}", info.sender, env.block.height);
    let code_ids = MSIG_CODE_IDS.load(deps.storage)?;

    let mut involved_addrs = vec![info.sender];
    for member in members.iter() {
        involved_addrs.push(deps.api.addr_validate(member.addr.as_str())?);
    }

    let msg = dao_interface::msg::InstantiateMsg {
        admin: None,
        name,
        description,
        image_url,
        automatically_add_cw20s: false,
        automatically_add_cw721s: false,
        voting_module_instantiate_info: ModuleInstantiateInfo {
            code_id: code_ids.voting,
            msg: to_json_binary(&cw4_voting::msg::InstantiateMsg {
                cw4_group_code_id: code_ids.cw4,
                initial_members: members,
            })?,
            admin: Some(Address {
                addr: env.contract.address.to_string(),
            }),
            label: format!("{}-voting-module", label),
        },
        proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
            code_id: code_ids.proposal,
            msg: to_json_binary(&dao_proposal_single::msg::InstantiateMsg {
                threshold: Threshold::ThresholdQuorum {
                    threshold: PercentageThreshold::Majority {},
                    quorum: PercentageThreshold::Majority {},
                },
                max_voting_period: Duration::Time(max_voting_period),
                min_voting_period: Some(Duration::Time(min_voting_period)),
                only_members_execute: true,
                allow_revoting: false,
                pre_propose_info: PreProposeInfo::ModuleMayPropose {
                    info: ModuleInstantiateInfo {
                        code_id: code_ids.pre_proposal,
                        msg: to_json_binary(&dao_pre_propose_base::msg::InstantiateMsg {
                            deposit_info: None,
                            open_proposal_submission: false,
                            extension: Empty {},
                        })?,
                        admin: Some(Address {
                            addr: env.contract.address.to_string(),
                        }),
                        label: format!("{}-pre-proposal-module", label),
                    },
                },
                close_proposal_on_execution_failure: true,
            })?,
            admin: Some(Address {
                addr: env.contract.address.to_string(),
            }),
            label: format!("{}-proposal-module", label),
        }],
        initial_items: None,
        dao_uri: None,
    };

    if PENDING_MSIG.exists(deps.storage) {
        return Err(ContractError::UnexpectedDoubleTx {});
    }

    PENDING_MSIG.save(deps.storage, &(involved_addrs, env.block.height))?;

    // Temporarily set the contract's admin to be the smart contract to setup some information
    Ok(Response::default().add_submessage(SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: code_ids.main,
            msg: to_json_binary(&msg)?,
            funds: vec![],
            label,
        },
        0,
    )))
}
