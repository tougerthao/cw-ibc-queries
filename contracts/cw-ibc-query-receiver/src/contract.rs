use cosmwasm_std::{
    entry_point, from_binary, to_binary, BankMsg, Deps, DepsMut, Env, IbcPacketAckMsg, MessageInfo,
    QueryResponse, Response, StdResult,
};
use cw_ibc_query::{PacketMsg, ReceiveIbcResponseMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{IbcQueryResultResponse, LATEST_QUERIES};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    // Do nothing for now
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cw_utils::nonpayable(&info)?;
    match msg {
        ExecuteMsg::ReceiveIbcResponse(ReceiveIbcResponseMsg { msg }) => {
            execute_receive(deps, env, info, msg)
        }
    }
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: IbcPacketAckMsg,
) -> Result<Response, ContractError> {
    //send funds to relayer
    let relayer_addr = msg.relayer.to_string();
    deps.api.addr_validate(&relayer_addr)?;
    let bank_msg = BankMsg::Send {
        to_address: relayer_addr.clone(),
        amount: info.funds,
    };
    // which local channel was this packet send from
    let channel_id = msg.original_packet.src.channel_id.clone();
    // store IBC response for later querying from the smart contract??
    LATEST_QUERIES.save(
        deps.storage,
        &channel_id,
        &IbcQueryResultResponse {
            last_update_time: env.block.time,
            response: msg,
        },
    )?;

    Ok(Response::default().add_message(bank_msg))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::LatestQueryResult { channel_id } => {
            to_binary(&query_latest_ibc_query_result(deps, channel_id)?)
        }
    }
}

fn query_latest_ibc_query_result(
    deps: Deps,
    channel_id: String,
) -> StdResult<IbcQueryResultResponse> {
    let results = LATEST_QUERIES.load(deps.storage, &channel_id)?;
    Ok(results)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;

    const RECEIVER: &str = "receiver";

    fn setup(deps: DepsMut) -> StdResult<Response> {
        let msg = InstantiateMsg {};
        let info = mock_info(RECEIVER, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let res = setup(deps.as_mut()).unwrap();
        println!("RES: {:?}", res.clone());
        assert_eq!(0, res.messages.len())
    }

    //TO DO: add more unit tests to test the execute function
}
