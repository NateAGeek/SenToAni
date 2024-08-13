use std::pin::Pin;

use bytemuck::Pod;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SizedSample;

use ffmpeg_next::subtitle::Text;
use ffmpeg_next::Packet;
use futures::future::OptionFuture;
use futures::FutureExt;
use ringbuf::ring_buffer::{RbRef, RbWrite};
use ringbuf::HeapRb;
use std::future::Future;

use super::ControlCommand;

pub struct SubtitlesPlaybackThread {
    control_sender: smol::channel::Sender<ControlCommand>,
    packet_sender: smol::channel::Sender<ffmpeg_next::codec::packet::packet::Packet>,
    receiver_thread: Option<std::thread::JoinHandle<()>>,
}

impl SubtitlesPlaybackThread {
    pub fn start(
        stream: &ffmpeg_next::format::stream::Stream,
        mut subtitle_frame_callback: Box<dyn FnMut(&ffmpeg_next::Packet) + Send>,
    ) -> Result<Self, anyhow::Error> {
        let (control_sender, control_receiver) = smol::channel::unbounded();

        let (packet_sender, packet_receiver) = smol::channel::bounded::<Packet>(128);

        let decoder_context = ffmpeg_next::codec::Context::from_parameters(stream.parameters())?;
        let mut packet_decoder = decoder_context.decoder().subtitle()?;

        let receiver_thread =
            std::thread::Builder::new().name("subtitle playback thread".into()).spawn(move || {
                smol::block_on(async move {
                    let packet_receiver_impl = async {
                        loop {
                            let Ok(packet) = packet_receiver.recv().await else { break };

                            // TODO: Properly await until we get a new packet with data to call the callback... Maybe something is built in...
                            // Or just wait util data.len() > 0?
                            println!("Received subtitle packet {:?}", packet.data().unwrap());
                            
                            smol::future::yield_now().await;

                            subtitle_frame_callback(&packet);

                            // while packet_decoder.decode(&packet, &mut subtitles).is_ok() {

                            //     // TODO: Report this as a bug, getting seg fault on reading utf8 string as unchecked str conversion? Maybe?
                            //     // https://ffmpeg.org/doxygen/trunk/structAVSubtitleRect.html#a893b1c87ee3d1816a0149ab3005fdd9e
                            //     // Per the docs, needs to be 0 terminated... Looks like base on the data, it is not 0 terminated...
                            //     // unsafe {
                            //     //     println!("Subtitles count: {}", subtitles.rects().len());
                            //     //     for rect in subtitles.rects() {
                            //     //         let text = Text::wrap(rect.as_ptr());
                            //     //         println!("Subtitle text: {:?}", rect.as_ptr());
                            //     //     }
                            //     // }
                            // }
                        }
                    }
                    .fuse()
                    .shared();
                
                    let mut playing = true;

                    loop {
                        let packet_receiver: OptionFuture<_> =
                            if playing { Some(packet_receiver_impl.clone()) } else { None }.into();

                        smol::pin!(packet_receiver);

                        futures::select! {
                            _ = packet_receiver => {},
                            received_command = control_receiver.recv().fuse() => {
                                match received_command {
                                    Ok(ControlCommand::Pause) => {
                                        playing = false;
                                    }
                                    Ok(ControlCommand::Play) => {
                                        playing = true;
                                    }
                                    Err(_) => {
                                        // Channel closed -> quit
                                        return;
                                    }
                                }
                            }
                        }
                    }
                })
            })?;

        Ok(Self { control_sender, packet_sender, receiver_thread: Some(receiver_thread) })
    }

    pub async fn receive_packet(&self, packet: ffmpeg_next::codec::packet::packet::Packet) -> bool {
        match self.packet_sender.send(packet).await {
            Ok(_) => return true,
            Err(smol::channel::SendError(_)) => return false,
        }
    }

    pub async fn send_control_message(&self, message: ControlCommand) {
        self.control_sender.send(message).await.unwrap();
    }
}

impl Drop for SubtitlesPlaybackThread {
    fn drop(&mut self) {
        self.control_sender.close();
        if let Some(receiver_join_handle) = self.receiver_thread.take() {
            receiver_join_handle.join().unwrap();
        }
    }
}