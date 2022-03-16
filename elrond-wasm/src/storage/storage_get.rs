use core::marker::PhantomData;

use crate::{
    api::{ErrorApi, ErrorApiImpl, ManagedTypeApi, StorageReadApi, StorageReadApiImpl},
    err_msg,
    types::{
        BigInt, BigUint, ManagedBuffer, ManagedBufferNestedDecodeInput, ManagedRef, ManagedType,
    },
};
use alloc::boxed::Box;
use elrond_codec::*;

use super::StorageKey;

struct StorageGetInput<'k, A>
where
    A: StorageReadApi + ManagedTypeApi + ErrorApi + 'static,
{
    key: ManagedRef<'k, A, StorageKey<A>>,
}

impl<'k, A> StorageGetInput<'k, A>
where
    A: StorageReadApi + ManagedTypeApi + ErrorApi + 'static,
{
    #[inline]
    fn new(key: ManagedRef<'k, A, StorageKey<A>>) -> Self {
        StorageGetInput { key }
    }

    fn to_managed_buffer(&self) -> ManagedBuffer<A> {
        let mbuf_handle = A::storage_read_api_impl()
            .storage_load_managed_buffer_raw(self.key.buffer.get_raw_handle());
        ManagedBuffer::from_raw_handle(mbuf_handle)
    }

    fn to_big_uint(&self) -> BigUint<A> {
        BigUint::from_bytes_be_buffer(&self.to_managed_buffer())
    }

    fn to_big_int(&self) -> BigInt<A> {
        BigInt::from_signed_bytes_be_buffer(&self.to_managed_buffer())
    }

    fn load_len_managed_buffer(&self) -> usize {
        A::storage_read_api_impl().storage_load_managed_buffer_len(self.key.buffer.get_raw_handle())
    }
}

impl<'k, A> TopDecodeInput for StorageGetInput<'k, A>
where
    A: StorageReadApi + ManagedTypeApi + ErrorApi + 'static,
{
    type NestedBuffer = ManagedBufferNestedDecodeInput<A>;

    fn byte_len(&self) -> usize {
        self.load_len_managed_buffer()
    }

    fn into_boxed_slice_u8(self) -> Box<[u8]> {
        let key_bytes = self.key.to_boxed_bytes();
        A::storage_read_api_impl()
            .storage_load_boxed_bytes(key_bytes.as_slice())
            .into_box()
    }

    #[inline]
    fn into_max_size_buffer<H, const MAX_LEN: usize>(
        self,
        buffer: &mut [u8; MAX_LEN],
        h: H,
    ) -> Result<&[u8], H::HandledErr>
    where
        H: DecodeErrorHandler,
    {
        self.to_managed_buffer().into_max_size_buffer(buffer, h)
    }

    #[inline]
    fn supports_specialized_type<T: TryStaticCast>() -> bool {
        T::type_eq::<ManagedBuffer<A>>() || T::type_eq::<BigUint<A>>() || T::type_eq::<BigInt<A>>()
    }

    #[inline]
    fn into_specialized<T, H>(self, h: H) -> Result<T, H::HandledErr>
    where
        T: TryStaticCast,
        H: DecodeErrorHandler,
    {
        if let Some(result) = try_execute_then_cast(|| self.to_managed_buffer()) {
            Ok(result)
        } else if let Some(result) = try_execute_then_cast(|| self.to_big_uint()) {
            Ok(result)
        } else if let Some(result) = try_execute_then_cast(|| self.to_big_int()) {
            Ok(result)
        } else {
            Err(h.handle_error(DecodeError::UNSUPPORTED_OPERATION))
        }
    }

    fn into_nested_buffer(self) -> Self::NestedBuffer {
        ManagedBufferNestedDecodeInput::new(self.to_managed_buffer())
    }
}

pub fn storage_get<A, T>(key: ManagedRef<'_, A, StorageKey<A>>) -> T
where
    T: TopDecode,
    A: StorageReadApi + ManagedTypeApi + ErrorApi,
{
    let Ok(value) = T::top_decode_or_handle_err(
        StorageGetInput::new(key),
        StorageGetErrorHandler::<A>::default(),
    );
    value
}

/// Useful for storage mappers.
/// Also calls to it generated by macro.
pub fn storage_get_len<A>(key: ManagedRef<'_, A, StorageKey<A>>) -> usize
where
    A: StorageReadApi + ManagedTypeApi + ErrorApi,
{
    A::storage_read_api_impl().storage_load_managed_buffer_len(key.get_raw_handle())
}

/// Will immediately end the execution when encountering the first decode error, via `signal_error`.
/// Because its handled error type is the never type, when compiled,
/// the codec will return the value directly, without wrapping it in a Result.
#[derive(Clone)]
struct StorageGetErrorHandler<M>
where
    M: ManagedTypeApi + ErrorApi,
{
    _phantom: PhantomData<M>,
}

impl<M> Copy for StorageGetErrorHandler<M> where M: ManagedTypeApi + ErrorApi {}

impl<M> Default for StorageGetErrorHandler<M>
where
    M: ManagedTypeApi + ErrorApi,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<M> DecodeErrorHandler for StorageGetErrorHandler<M>
where
    M: ManagedTypeApi + ErrorApi,
{
    type HandledErr = !;

    fn handle_error(&self, err: DecodeError) -> Self::HandledErr {
        let mut message_buffer = ManagedBuffer::<M>::new_from_bytes(err_msg::STORAGE_DECODE_ERROR);
        message_buffer.append_bytes(err.message_bytes());
        M::error_api_impl().signal_error_from_buffer(message_buffer.get_raw_handle())
    }
}
