use futures::{FutureExt, StreamExt};
use rustdds::StatusEvented;
use rustdds::{
    policy::Reliability, with_key::DataWriter, DataWriterStatus, DomainParticipantBuilder, Keyed,
    QosPolicyBuilder, TopicKind,
};
use serde::{Deserialize, Serialize};
use smol::channel::{Receiver, Sender};
use smol::Timer;
use std::time::Duration;
use tonic::Status;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManualTrackCreate {
    pub user_id: i32,
    pub message: String,
}

impl Keyed for ManualTrackCreate {
    type K = i32;
    fn key(&self) -> Self::K {
        self.user_id
    }
}

#[derive(Debug)]
pub struct AddManualTrackPublisher {
    sender: Sender<ManualTrackCreate>,
}

impl Default for AddManualTrackPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl AddManualTrackPublisher {
    pub fn new() -> Self {
        let (sender, receiver) = smol::channel::bounded(100); // Buffer untuk menerima data
        smol::spawn(async move {
            Self::run(receiver).await;
        })
        .detach();
        Self { sender }
    }

    async fn run(receiver: Receiver<ManualTrackCreate>) {
        let domain_participant = DomainParticipantBuilder::new(0)
            .build()
            .unwrap_or_else(|e| panic!("DomainParticipant construction failed: {e:?}"));

        let qos = QosPolicyBuilder::new()
            .reliability(Reliability::Reliable {
                max_blocking_time: rustdds::Duration::from_secs(1),
            })
            .build();

        let topic = domain_participant
            .create_topic(
                "ManualTrackCreate_Message".to_string(),
                "ManualTrackCreate::Message".to_string(),
                &qos,
                TopicKind::WithKey,
            )
            .unwrap_or_else(|e| panic!("create_topic failed: {e:?}"));

        let publisher = domain_participant.create_publisher(&qos).unwrap();
        let writer = publisher
            .create_datawriter_cdr::<ManualTrackCreate>(&topic, None)
            .unwrap();

        smol::block_on(async {
            let mut datawriter_event_stream = writer.as_async_status_stream();

            let mut match_timeout_timer =
                futures::FutureExt::fuse(Timer::after(Duration::from_secs(10)));

            println!("Publisher is ready.");
            loop {
                futures::select! {
                    data = receiver.recv().fuse() => {
                        match data {
                            Ok(hello_message) => {
                                println!("Publishing: {:?}", hello_message);
                                writer.async_write(hello_message.clone(), None).await.unwrap();
                            }
                            Err(_) => break, // Receiver closed
                        }
                    }
                    event = datawriter_event_stream.select_next_some() => {
                        println!("DataWriter event: {:?}", event);
                    }
                }
            }
        });
    }

    // publish message
    pub async fn publish_message(&self, message: ManualTrackCreate) -> Result<(), Status> {
        self.sender
            .send(message)
            .await
            .map_err(|_| Status::internal("Failed to send message"))
    }
}