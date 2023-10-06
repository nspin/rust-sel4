use sel4_microkit::MessageInfo;
use sel4_microkit_message::MessageInfoExt as _;

use sel4_hal_adapters::smoltcp::phy::{MacAddress, Request, GetMacAddressResponse};

pub struct NetClient {
    channel: sel4_microkit::Channel,
}

impl NetClient {
    pub fn new(channel: sel4_microkit::Channel) -> Self {
        Self { channel }
    }

    pub fn get_mac_address(&self) -> MacAddress {
        let req = Request::GetMacAddress;
        let resp: GetMacAddressResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.mac_address
    }
}
