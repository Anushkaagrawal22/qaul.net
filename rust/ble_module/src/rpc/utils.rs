// Copyright (c) 2023 Open Community Project Association https://ocpa.ch
// This software is published under the AGPLv3 license.

use async_std::channel::Sender;
use bluer::{Adapter, Address};

use super::proto_sys::{self, BleDeviceDiscovered, BleError, BleStartResult, BleDeviceUnavailable};

#[derive(Clone)]
pub struct BleResultSender(Sender<Vec<u8>>);

impl BleResultSender {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        BleResultSender(sender)
    }

    pub fn send_ble_sys_msg(&mut self, msg: proto_sys::ble::Message) {
        let mut buf = Vec::with_capacity(msg.encoded_len());
        msg.encode(&mut buf);
        if let Err(err) = self.0.try_send(buf) {
            log::error!("{:?}", err);
        }
    }

    pub fn send_result_already_running(&mut self) {
        self.send_ble_sys_msg(proto_sys::ble::Message::StartResult(BleStartResult {
            success: true,
            error_reason: BleError::UnknownError.into(),
            error_message: "Received start request, but BLE service is already running!".into(),
        }));
    }

    pub fn send_result_not_running(&mut self) {
        self.send_ble_sys_msg(proto_sys::ble::Message::StartResult(BleStartResult {
            success: true,
            error_reason: BleError::UnknownError.into(),
            error_message: "Received direct send request, but BLE service is not yet running!"
                .into(),
        }));
    }

    pub fn send_start_successful(&mut self) {
        self.send_ble_sys_msg(proto_sys::ble::Message::StartResult(BleStartResult {
            success: true,
            error_reason: 0,
            error_message: "".into(),
        }))
    }

    pub fn send_device_found(&mut self, qaul_id: Vec<u8>, rssi: i32) {
        self.send_ble_sys_msg(proto_sys::ble::Message::DeviceDiscovered(
            BleDeviceDiscovered { qaul_id, rssi },
        ))
    }

    pub fn send_device_unavailable(&mut self, qaul_id: Vec<u8>, adapter: Adapter, mac_address: Address) {
        async_std::task::spawn(async move {
            let _ = adapter.remove_device(mac_address).await;
        });
        self.send_ble_sys_msg(proto_sys::ble::Message::DeviceUnavailable(
            BleDeviceUnavailable { qaul_id },
        ))
    }

    pub fn send_direct_received(&mut self, from: Vec<u8>, data: Vec<u8>) {
        self.send_ble_sys_msg(proto_sys::ble::Message::DirectReceived(
            proto_sys::BleDirectReceived { from, data },
        ))
    }

    pub fn send_direct_send_success(&mut self, id: Vec<u8>) {
        self.send_ble_sys_msg(proto_sys::ble::Message::DirectSendResult(
            proto_sys::BleDirectSendResult {
                id,
                success: true,
                error_message: "".into(),
            },
        ))
    }

    pub fn send_direct_send_error(&mut self, id: Vec<u8>, error_message: String) {
        self.send_ble_sys_msg(proto_sys::ble::Message::DirectSendResult(
            proto_sys::BleDirectSendResult {
                id,
                success: false,
                error_message,
            },
        ))
    }

    pub fn send_stop_unsuccessful(&mut self, error_message: String) {
        self.send_ble_sys_msg(proto_sys::ble::Message::StopResult(
            proto_sys::BleStopResult {
                success: false,
                error_reason: 0,
                error_message,
            },
        ))
    }

    pub fn send_stop_successful(&mut self) {
        self.send_ble_sys_msg(proto_sys::ble::Message::StopResult(
            proto_sys::BleStopResult {
                success: true,
                error_reason: 0,
                error_message: "".into(),
            },
        ))
    }
}
