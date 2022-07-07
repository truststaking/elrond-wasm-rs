use crate::{
    num_bigint::{BigInt, BigUint},
    tx_mock::TxPanic,
    DebugApi,
};
use alloc::vec::Vec;
use elrond_wasm::api::{EndpointArgumentApi, EndpointArgumentApiImpl, ManagedBufferApi};
use num_traits::cast::ToPrimitive;

impl EndpointArgumentApi for DebugApi {
    type EndpointArgumentApiImpl = DebugApi;

    fn argument_api_impl() -> Self::EndpointArgumentApiImpl {
        DebugApi::new_from_static()
    }
}

impl DebugApi {
    fn get_argument_vec_u8(&self, arg_index: i32) -> Vec<u8> {
        let arg_idx_usize = arg_index as usize;
        assert!(
            arg_idx_usize < self.input_ref().args.len(),
            "Tx arg index out of range"
        );
        self.input_ref().args[arg_idx_usize].clone()
    }
}

/// Interface to only be used by code generated by the macros.
/// The smart contract code doesn't have access to these methods directly.
impl EndpointArgumentApiImpl for DebugApi {
    fn get_num_arguments(&self) -> i32 {
        self.input_ref().args.len() as i32
    }

    fn get_argument_len(&self, arg_index: i32) -> usize {
        let arg = self.get_argument_vec_u8(arg_index);
        arg.len()
    }

    fn load_argument_managed_buffer(&self, arg_index: i32, dest: Self::ManagedBufferHandle) {
        let arg_bytes = self.get_argument_vec_u8(arg_index);
        self.mb_overwrite(dest, arg_bytes.as_slice());
    }

    fn get_argument_i64(&self, arg_index: i32) -> i64 {
        // specific implementation provided, in order to simulate the VM error (status 10 instead of 4)
        let bytes = self.get_argument_vec_u8(arg_index);
        let bi = BigInt::from_signed_bytes_be(&bytes);
        if let Some(v) = bi.to_i64() {
            v
        } else {
            std::panic::panic_any(TxPanic {
                status: 10,
                message: "argument out of range".to_string(),
            })
        }
    }

    fn get_argument_u64(&self, arg_index: i32) -> u64 {
        // specific implementation provided, in order to simulate the VM error (status 10 instead of 4)
        let bytes = self.get_argument_vec_u8(arg_index);
        let bu = BigUint::from_bytes_be(&bytes);
        if let Some(v) = bu.to_u64() {
            v
        } else {
            std::panic::panic_any(TxPanic {
                status: 10,
                message: "argument out of range".to_string(),
            })
        }
    }
}
