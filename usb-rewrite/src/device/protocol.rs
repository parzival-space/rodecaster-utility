use flume::{Receiver, RecvError, Sender};
use futures::select;
use log::{debug, error, trace, warn};
use std::future::Future;
use hidapi::DeviceInfo;
use crate::device::io::{RodeCasterDeviceIOCommand, RodeCasterDeviceIOEvent};
use crate::protocol::packet::handshake::HandshakePacket;
use crate::protocol::packet::{Packet};
use crate::protocol::packet::update::UpdatePacket;
use crate::protocol::{Deserialize, Serialize};

pub(crate) enum RodeCasterDeviceProtocolHandlerEvent {

}

pub(crate) enum RodeCasterDeviceProtocolHandlerCommand {
    SendPacket(Box<dyn Packet>),
    Close,
}

pub(crate) trait RodeCasterDeviceProtocolHandler {
    fn spawn_protocol_handler(
        device_info: DeviceInfo,
        sender: Sender<RodeCasterDeviceProtocolHandlerEvent>,
        receiver: Receiver<RodeCasterDeviceProtocolHandlerCommand>,
        io_sender: Sender<RodeCasterDeviceIOCommand>,
        io_receiver: Receiver<RodeCasterDeviceIOEvent>,
    ) {
        let sender = sender.clone();
        let receiver = receiver.clone();
        let io_sender = io_sender.clone();
        let io_receiver = io_receiver.clone();

        #[cfg(feature = "tokio")]
        {
            use tokio::runtime::Handle;
            if let Ok(handle) = Handle::try_current() {
                debug!("Starting protocol handler task");
                handle.spawn(async move {
                    Self::run_handler(
                        sender,
                        receiver,
                        io_sender,
                        io_receiver
                    ).await
                });
                return;
            }
        }

        // fallback to threads
        // try to find name in this order: product string > serial number > device path > "unknown"
        let device_name = device_info.product_string().unwrap_or(
            device_info.serial_number().unwrap_or(
                device_info.path().to_str().unwrap_or("rodecaster")));

        use std::thread::Builder;
        let _ = Builder::new()
            .name(format!("{}-rx", device_name))
            .spawn(move || {
                futures::executor::block_on(Self::run_handler(
                    sender,
                    receiver,
                    io_sender,
                    io_receiver
                ));
            });
    }

    fn run_handler(
        sender: Sender<RodeCasterDeviceProtocolHandlerEvent>,
        receiver: Receiver<RodeCasterDeviceProtocolHandlerCommand>,
        io_sender: Sender<RodeCasterDeviceIOCommand>,
        io_receiver: Receiver<RodeCasterDeviceIOEvent>,
    ) -> impl Future<Output = ()> + Send {
        async move {
            // initialize device by sending handshake command
            // todo: if this fails because of delayed startups, try to resend it
            match HandshakePacket::default().serialize() {
                Ok(bytes) => {
                    let _ = io_sender.send_async(RodeCasterDeviceIOCommand::SendPacket(bytes)).await;
                }
                Err(err) => {
                    error!("Error serializing packet: {}", err);
                    warn!("Failed to serialize handshake packet, the device will not respond to commands!");
                }
            }


            loop {
                select! {
                    result = receiver.recv_async() => match result {
                        Ok(command) => match command {
                            RodeCasterDeviceProtocolHandlerCommand::SendPacket(packet) => {
                                let Ok(packet_bytes) = packet.serialize() else {
                                    warn!("Failed to serialize packet {:?}", packet);
                                    continue;
                                };

                                let _ = io_sender.send_async(RodeCasterDeviceIOCommand::SendPacket(packet_bytes));
                            }
                            RodeCasterDeviceProtocolHandlerCommand::Close => {
                                break;
                            }
                        }
                        Err(err) => match err {
                            RecvError::Disconnected => {
                                break;
                            }
                        }
                    },
                    result = io_receiver.recv_async() => match result {
                        Ok(event) => match event {
                            RodeCasterDeviceIOEvent::PacketReceived(packet_bytes) => {
                                if let Ok((_, packet)) = UpdatePacket::deserialize(&packet_bytes) {
                                    trace!("Received update packet: {:?}", packet);
                                } else {
                                    warn!("Failed to parse packet: {:?}", packet_bytes);
                                }
                            }
                            RodeCasterDeviceIOEvent::TXError(err) => {
                                error!("TX error: {:?}", err);
                            }
                            RodeCasterDeviceIOEvent::RXError(err) => {
                                error!("RX error: {:?}", err);
                            }
                        }
                        Err(err) => match err {
                            RecvError::Disconnected => {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}