use std::{cell::RefCell, rc::Rc};

use crate::{
    tx_mock::{TxContext, TxContextRef, TxInput, TxOutput, TxResult},
    world_mock::{AccountData, BlockchainMock, BlockchainMockError},
    AsyncCallTxData, ContractMap, DebugApi,
};

use super::execute_tx_context;

pub fn sc_create(
    tx_input: TxInput,
    contract_path: &[u8],
    state: &mut Rc<BlockchainMock>,
    contract_map: &ContractMap<DebugApi>,
) -> Result<(TxResult, Option<AsyncCallTxData>), BlockchainMockError> {
    let tx_context = TxContextRef::new(tx_input, state.clone());
    let tx_input_ref = &*tx_context.tx_input_box;

    tx_context
        .blockchain_cache
        .increase_acount_nonce(&tx_input_ref.from);
    tx_context
        .blockchain_cache
        .subtract_egld_balance(&tx_input_ref.from, &tx_input_ref.egld_value)?;
    tx_context.blockchain_cache.subtract_tx_gas(
        &tx_input_ref.from,
        tx_input_ref.gas_limit,
        tx_input_ref.gas_price,
    );
    let new_address =
        tx_context.create_new_contract(contract_path.to_vec(), tx_input_ref.from.clone());
    tx_context
        .blockchain_cache
        .increase_egld_balance(&new_address, &tx_input_ref.egld_value);

    let tx_result = execute_tx_context(tx_context.clone(), contract_map);

    let blockchain_updates = tx_context.into_blockchain_updates();
    blockchain_updates.apply(Rc::get_mut(state).unwrap());

    // let from = tx_input.from.clone();
    // let to = tx_input.to.clone();
    // let call_value = tx_input.egld_value.clone();
    // let blockchain_info = state.create_tx_info(&to);

    // state.subtract_egld_balance(&from, &call_value)?;
    // state.subtract_tx_gas(&from, tx_input.gas_limit, tx_input.gas_price);

    // let tx_context = DebugApi::new(blockchain_info, tx_input.clone(), TxOutput::default());
    // let mut tx_output = execute_contract_endpoint(tx_context, contract_path, contract_map);

    // if tx_result.result_status == 0 {
    //     let new_address = state.create_account_after_deploy(
    //         &tx_input,
    //         tx_output.contract_storage,
    //         contract_path.to_vec(),
    //     );
    //     state.send_balance(
    //         &new_address,
    //         tx_output.send_balance_list.as_slice(),
    //         &mut tx_result.result_logs,
    //     )?;
    // } else {
    //     state.increase_egld_balance(&from, &call_value);
    // }

    Ok((tx_result, None))
}
