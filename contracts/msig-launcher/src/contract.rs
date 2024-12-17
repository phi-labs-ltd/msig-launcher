#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Bound;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::execute_instantiate;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, PageResult, QueryMsg, PAGINATION_DEFAULT,
    PAGINATION_LIMIT,
};
use crate::state::{MSIG, MSIG_CODE_IDS};
/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:msig-launcher";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    MSIG_CODE_IDS.save(deps.storage, &msg.code_ids)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Instantiate {
            name,
            description,
            image_url,
            max_voting_period,
            min_voting_period,
            members,
        } => execute_instantiate(
            deps,
            env,
            info,
            name,
            description,
            image_url,
            min_voting_period,
            max_voting_period,
            members,
        ),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::MSigs { pagination } => {
            let limit = pagination
                .limit
                .unwrap_or(PAGINATION_DEFAULT)
                .min(PAGINATION_LIMIT) as usize;
            let start_at = pagination.start_at.unwrap_or(0);

            let mut results = vec![];
            let mut page = None;

            for (i, res) in MSIG
                .range(
                    deps.storage,
                    Some(Bound::inclusive((pagination.user.clone(), start_at))),
                    Some(Bound::inclusive((pagination.user.clone(), u64::MAX))),
                    Order::Ascending,
                )
                .take(limit + 1)
                .enumerate()
            {
                let ((_, height), result) = res?;

                // If we got the expected amount of items + 1
                // then theres more to query the next time
                if i == limit {
                    page = Some(height);
                } else {
                    results.push(result);
                }
            }

            to_json_binary(&PageResult {
                data: results,
                next: page,
            })
        }
        QueryMsg::CodeIds {} => to_json_binary(&MSIG_CODE_IDS.load(deps.storage)?),
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, PageResult, Pagination, QueryMsg};
    use crate::state::MSigCodeIds;
    use archway_test_tube::module::{Module, Wasm};
    use archway_test_tube::test_tube::Account;
    use archway_test_tube::{arch, ArchwayApp};
    use cosmwasm_std::Addr;
    use cw4::Member;

    #[test]
    fn instantiate() {
        let app = ArchwayApp::default();

        let accounts = app.init_accounts(&[arch(100)], 2).unwrap();
        let admin = accounts.get(0).unwrap();
        let user = accounts.get(1).unwrap();

        let wasm = Wasm::new(&app);
        let dao_core_file = std::fs::read("../../external_wasm/dao_dao_core.wasm").unwrap();
        let dao_core_id = wasm
            .store_code(&dao_core_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let voting_file = std::fs::read("../../external_wasm/cw4_voting.wasm").unwrap();
        let voting_id = wasm
            .store_code(&voting_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let proposal_file = std::fs::read("../../external_wasm/dao_proposal_single.wasm").unwrap();
        let proposal_id = wasm
            .store_code(&proposal_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let pre_propose_file =
            std::fs::read("../../external_wasm/dao_pre_propose_single.wasm").unwrap();
        let pre_propose_id = wasm
            .store_code(&pre_propose_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let cw4 = std::fs::read("../../external_wasm/cw4_group.wasm").unwrap();
        let cw4_id = wasm.store_code(&cw4, None, admin).unwrap().data.code_id;

        // Msig launcher
        let launcher =
            std::fs::read("../../target/wasm32-unknown-unknown/release/msig_launcher.wasm")
                .unwrap();
        let launcher_code_id = wasm
            .store_code(&launcher, None, admin)
            .unwrap()
            .data
            .code_id;

        let launcher = wasm
            .instantiate(
                launcher_code_id,
                &InstantiateMsg {
                    code_ids: MSigCodeIds {
                        main: dao_core_id,
                        voting: voting_id,
                        proposal: proposal_id,
                        pre_proposal: pre_propose_id,
                        cw4: cw4_id,
                    },
                },
                None,
                Some("label"),
                &[],
                admin,
            )
            .unwrap()
            .data
            .address;

        let member_accounts = app.init_accounts(&[arch(100)], 3).unwrap();
        let _res = wasm
            .execute(
                &launcher,
                &ExecuteMsg::Instantiate {
                    name: "SomeWallet".to_string(),
                    description: "SomeDescription".to_string(),
                    image_url: Some("Image".to_string()),
                    max_voting_period: 100,
                    min_voting_period: 0,
                    members: vec![
                        Member {
                            addr: member_accounts.get(0).unwrap().address(),
                            weight: 1,
                        },
                        Member {
                            addr: member_accounts.get(1).unwrap().address(),
                            weight: 1,
                        },
                        Member {
                            addr: member_accounts.get(2).unwrap().address(),
                            weight: 1,
                        },
                    ],
                },
                &[],
                user,
            )
            .unwrap();

        // Queried
        let res = wasm
            .query::<_, PageResult>(
                &launcher,
                &QueryMsg::MSigs {
                    pagination: Pagination {
                        user: Addr::unchecked(user.address()),
                        limit: None,
                        start_at: None,
                    },
                },
            )
            .unwrap();
        let main = res.data.get(0).unwrap();

        for addr in member_accounts {
            let res = wasm
                .query::<_, PageResult>(
                    &launcher,
                    &QueryMsg::MSigs {
                        pagination: Pagination {
                            user: Addr::unchecked(addr.address()),
                            limit: None,
                            start_at: None,
                        },
                    },
                )
                .unwrap();
            let msig = res.data.get(0).unwrap();

            assert_eq!(msig, main);
        }
    }

    #[test]
    fn query() {
        let app = ArchwayApp::default();

        let accounts = app.init_accounts(&[arch(100)], 2).unwrap();
        let admin = accounts.get(0).unwrap();
        let user = accounts.get(1).unwrap();

        let wasm = Wasm::new(&app);
        let dao_core_file = std::fs::read("../../external_wasm/dao_dao_core.wasm").unwrap();
        let dao_core_id = wasm
            .store_code(&dao_core_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let voting_file = std::fs::read("../../external_wasm/cw4_voting.wasm").unwrap();
        let voting_id = wasm
            .store_code(&voting_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let proposal_file = std::fs::read("../../external_wasm/dao_proposal_single.wasm").unwrap();
        let proposal_id = wasm
            .store_code(&proposal_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let pre_propose_file =
            std::fs::read("../../external_wasm/dao_pre_propose_single.wasm").unwrap();
        let pre_propose_id = wasm
            .store_code(&pre_propose_file, None, admin)
            .unwrap()
            .data
            .code_id;
        let cw4 = std::fs::read("../../external_wasm/cw4_group.wasm").unwrap();
        let cw4_id = wasm.store_code(&cw4, None, admin).unwrap().data.code_id;

        // Msig launcher
        let launcher =
            std::fs::read("../../target/wasm32-unknown-unknown/release/msig_launcher.wasm")
                .unwrap();
        let launcher_code_id = wasm
            .store_code(&launcher, None, admin)
            .unwrap()
            .data
            .code_id;

        let launcher = wasm
            .instantiate(
                launcher_code_id,
                &InstantiateMsg {
                    code_ids: MSigCodeIds {
                        main: dao_core_id,
                        voting: voting_id,
                        proposal: proposal_id,
                        pre_proposal: pre_propose_id,
                        cw4: cw4_id,
                    },
                },
                None,
                Some("label"),
                &[],
                admin,
            )
            .unwrap()
            .data
            .address;

        let member_accounts = app.init_accounts(&[arch(100)], 3).unwrap();
        let _res = wasm
            .execute(
                &launcher,
                &ExecuteMsg::Instantiate {
                    name: "SomeWallet".to_string(),
                    description: "SomeDescription".to_string(),
                    image_url: Some("Image".to_string()),
                    max_voting_period: 100,
                    min_voting_period: 0,
                    members: vec![
                        Member {
                            addr: member_accounts.get(0).unwrap().address(),
                            weight: 1,
                        },
                        Member {
                            addr: member_accounts.get(1).unwrap().address(),
                            weight: 1,
                        },
                        Member {
                            addr: member_accounts.get(2).unwrap().address(),
                            weight: 1,
                        },
                    ],
                },
                &[],
                user,
            )
            .unwrap();
        let _res = wasm
            .execute(
                &launcher,
                &ExecuteMsg::Instantiate {
                    name: "SomeOtherWallet".to_string(),
                    description: "SomeDescription".to_string(),
                    image_url: Some("Image".to_string()),
                    max_voting_period: 100,
                    min_voting_period: 0,
                    members: vec![
                        Member {
                            addr: member_accounts.get(0).unwrap().address(),
                            weight: 1,
                        },
                        Member {
                            addr: member_accounts.get(1).unwrap().address(),
                            weight: 1,
                        },
                        Member {
                            addr: member_accounts.get(2).unwrap().address(),
                            weight: 1,
                        },
                    ],
                },
                &[],
                user,
            )
            .unwrap();

        // Queried
        let res = wasm
            .query::<_, PageResult>(
                &launcher,
                &QueryMsg::MSigs {
                    pagination: Pagination {
                        user: Addr::unchecked(user.address()),
                        limit: None,
                        start_at: None,
                    },
                },
            )
            .unwrap();
        dbg!(&res);
        assert_eq!(res.data.len(), 2);
        assert!(res.next.is_none());

        let res = wasm
            .query::<_, PageResult>(
                &launcher,
                &QueryMsg::MSigs {
                    pagination: Pagination {
                        user: Addr::unchecked(user.address()),
                        limit: Some(1),
                        start_at: None,
                    },
                },
            )
            .unwrap();
        assert_eq!(res.data.len(), 1);
        assert!(res.next.is_some());
    }
}
