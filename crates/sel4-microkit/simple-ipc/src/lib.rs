//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use sel4_microkit_base::{
    Channel, MessageInfo, MessageLabel, MessageRegisterValue, with_msg_bytes, with_msg_bytes_mut,
};

const MAX_MESSAGE_LABEL: MessageLabel =
    !0 >> (size_of::<MessageInfo>() * 8 - MessageInfo::label_width());

pub const UNSPECIFIED_ERROR_MESSAGE_LABEL: MessageLabel = MAX_MESSAGE_LABEL;

pub fn send_unspecified_error() -> MessageInfo {
    MessageInfo::new(UNSPECIFIED_ERROR_MESSAGE_LABEL, 0)
}

pub fn try_send<T: Serialize>(val: T) -> Result<MessageInfo, SendError> {
    with_msg_bytes_mut(|buf| {
        let used = postcard::to_slice(&val, buf).map_err(SendError::PostcardError)?;
        let count = bytes_to_words(used.len());
        Ok(MessageInfo::new(0, count))
    })
}

pub fn send<T: Serialize>(val: T) -> MessageInfo {
    try_send(val).unwrap()
}

pub fn try_recv<T: for<'a> Deserialize<'a>>(msg_info: MessageInfo) -> Result<T, RecvError> {
    let label = msg_info.label();
    if label != 0 {
        return Err(RecvError::UnexpectedLabel { label });
    }
    with_msg_bytes(|buf| postcard::from_bytes(buf).map_err(RecvError::PostcardError))
}

pub fn recv<T: for<'a> Deserialize<'a>>(msg_info: MessageInfo) -> T {
    try_recv(msg_info).unwrap()
}

pub fn try_call<T: Serialize, U: for<'a> Deserialize<'a>>(
    channel: Channel,
    val: T,
) -> Result<U, CallError> {
    let req_msg_info = try_send(val).map_err(CallError::SendError)?;
    let resp_msg_info = channel.pp_call(req_msg_info);
    try_recv(resp_msg_info).map_err(CallError::RecvError)
}

pub fn call<T: Serialize, U: for<'a> Deserialize<'a>>(
    channel: Channel,
    val: T,
) -> Result<U, RecvError> {
    try_call(channel, val).map_err(|err| match err {
        CallError::SendError(err) => panic!("send error: {err}"),
        CallError::RecvError(err) => err,
    })
}

#[derive(Clone, Debug, Error)]
pub enum SendError {
    #[error("postcard error: {0}")]
    PostcardError(postcard::Error),
}

#[derive(Clone, Debug, Error)]
pub enum RecvError {
    #[error("unexpected label: {label}")]
    UnexpectedLabel { label: MessageLabel },
    #[error("postcard error: {0}")]
    PostcardError(postcard::Error),
}

#[derive(Clone, Debug, Error)]
pub enum CallError {
    #[error("send error: {0}")]
    SendError(SendError),
    #[error("recv error: {0}")]
    RecvError(RecvError),
}

// // //

fn bytes_to_words(num_bytes: usize) -> usize {
    num_bytes.div_ceil(size_of::<MessageRegisterValue>())
}
