use archway_proto::prost::{Message, Name};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::execute_instantiate;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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
            label,
            name,
            description,
            image_url,
            max_voting_period,
            members,
        } => execute_instantiate(
            deps,
            env,
            info,
            label,
            name,
            description,
            image_url,
            max_voting_period,
            members,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::MSig { label } => to_json_binary(&MSIG.load(deps.storage, label)?),
        QueryMsg::CodeIds {} => to_json_binary(&MSIG_CODE_IDS.load(deps.storage)?),
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{MSig, MSigCodeIds};
    use archway_test_tube::module::{Module, Wasm};
    use archway_test_tube::test_tube::Account;
    use archway_test_tube::{arch, ArchwayApp};
    use cw4::Member;
    use cw_multi_test::Executor;

    #[test]
    fn instantiate() {
        let mut app = ArchwayApp::default();

        let accounts = app.init_accounts(&[arch(100)], 2).unwrap();
        let admin = accounts.get(0).unwrap();
        let user = accounts.get(1).unwrap();

        let wasm = Wasm::new(&app);
        let dao_core_file = std::fs::read("../../external_wasm/dao_dao_core.wasm").unwrap();
        wasm.store_code(&dao_core_file, None, admin).unwrap();
        let voting_file = std::fs::read("../../external_wasm/cw4_voting.wasm").unwrap();
        wasm.store_code(&voting_file, None, admin).unwrap();
        let proposal_file = std::fs::read("../../external_wasm/dao_proposal_single.wasm").unwrap();
        wasm.store_code(&proposal_file, None, admin).unwrap();
        let pre_propose_file =
            std::fs::read("../../external_wasm/dao_pre_propose_single.wasm").unwrap();
        wasm.store_code(&pre_propose_file, None, admin).unwrap();
        let cw4 = std::fs::read("../../external_wasm/cw4_group.wasm").unwrap();
        wasm.store_code(&cw4, None, admin).unwrap();

        // Msig launcher
        let launcher = std::fs::read("../../artifacts/msig_launcher.wasm").unwrap();
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
                        main: 1,
                        voting: 2,
                        proposal: 3,
                        pre_proposal: 4,
                        cw4: 5,
                    },
                },
                None,
                Some("label".clone()),
                &[],
                admin,
            )
            .unwrap()
            .data
            .address;

        let member_accounts = app.init_accounts(&[arch(100)], 3).unwrap();
        let res = wasm
            .execute(
                &launcher,
                &ExecuteMsg::Instantiate {
                    label: "msig".to_string(),
                    name: "SomeWallet".to_string(),
                    description: "SomeDescription".to_string(),
                    image_url: Some("Image".to_string()),
                    max_voting_period: 100,
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

        // Get the last event which should be this contract's reply
        let relevant_event = res.events.last().unwrap();
        let mut event_iter = relevant_event.attributes.iter();
        // Skip the execution attr
        event_iter.next();

        // Queried
        let res = wasm
            .query::<_, MSig>(
                &launcher,
                &QueryMsg::MSig {
                    label: "msig".to_string(),
                },
            )
            .unwrap();

        let mut queried_attrs = vec![];
        res.append_attrs(&mut queried_attrs);

        for (a, b) in queried_attrs.iter().zip(event_iter) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.value, b.value);
        }
    }
}
