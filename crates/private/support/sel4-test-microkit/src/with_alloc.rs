
use core::error::Error;

use sel4_microkit::{Handler, ChannelSet, Event};


// pub fn upcast_handler<E: Error>(impl Handler<Error = E> + 'static) -> Box<dyn Handler<Error = Box<dyn Error>> + 'static> {

// }


struct DynErrorHandlerWrapper<T>(T);

impl<T: Handler + ?Sized> Handler for Box<T> {
    type Error = T::Error;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        (**self).notified(channels)
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        (**self).protected(channel, msg_info)
    }

    fn fault(
        &mut self,
        child: Child,
        msg_info: MessageInfo,
    ) -> Result<Option<MessageInfo>, Self::Error> {
        (**self).fault(child, msg_info)
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        (**self).take_deferred_action()
    }

    #[doc(hidden)]
    fn run(&mut self) -> Result<Never, Self::Error> {
        (**self).run()
    }
}
