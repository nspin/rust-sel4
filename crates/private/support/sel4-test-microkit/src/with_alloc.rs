use alloc::boxed::Box;
use core::error::Error;

use sel4_microkit::{Channel, ChannelSet, Child, DeferredAction, Handler, MessageInfo, Never};

// pub fn upcast_handler<E: Error>(impl Handler<Error = E> + 'static) -> Box<dyn Handler<Error = Box<dyn Error>> + 'static> {

// }

struct DynErrorHandlerWrapper<T>(T);

impl<T: Handler> Handler for DynErrorHandlerWrapper<T> {
    type Error = T::Error;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        self.0.notified(channels).map_err(Into::into)
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        self.0.protected(channel, msg_info).map_err(Into::into)
    }

    fn fault(
        &mut self,
        child: Child,
        msg_info: MessageInfo,
    ) -> Result<Option<MessageInfo>, Self::Error> {
        self.0.fault(child, msg_info).map_err(Into::into)
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        self.0.take_deferred_action()
    }

    #[doc(hidden)]
    fn run(&mut self) -> Result<Never, Self::Error> {
        self.0.run().map_err(Into::into)
    }
}
