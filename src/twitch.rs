use crate::map::OsuMapSet;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

pub struct OsuRelatedTwitchMessage {
    pub sender: String,
    pub message: String,
    pub maps: Vec<OsuMapSet>,
}

pub struct TwitchWithOsu {
    channel: String,

    client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,

    incoming_messages: Option<UnboundedReceiver<ServerMessage>>,
    sender: Option<UnboundedSender<OsuRelatedTwitchMessage>>,
}

impl TwitchWithOsu {
    pub fn new(channel: String) -> (TwitchWithOsu, UnboundedReceiver<OsuRelatedTwitchMessage>) {
        let config = ClientConfig::default();
        let (incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        let (sender, receiver) = mpsc::unbounded_channel::<OsuRelatedTwitchMessage>();

        (
            TwitchWithOsu {
                channel,
                client,
                incoming_messages: Some(incoming_messages),
                sender: Some(sender),
            },
            receiver,
        )
    }

    pub async fn start(&mut self) -> Option<JoinHandle<()>> {
        let Some(incoming_messages) = self.incoming_messages.take() else {
            return None;
        };

        let Some(sender) = self.sender.take() else {
            return None;
        };

        let join_handle = tokio::spawn(async move {
            TwitchWithOsu::process_messages(incoming_messages, sender).await;
        });

        self.client.join(self.channel.to_owned()).unwrap();

        Some(join_handle)
    }

    async fn process_messages(
        mut incoming_messages: UnboundedReceiver<ServerMessage>,
        sender: UnboundedSender<OsuRelatedTwitchMessage>,
    ) {
        while let Some(message) = incoming_messages.recv().await {
            let ServerMessage::Privmsg(msg) = message else {
                continue;
            };

            let maps: Vec<OsuMapSet> = msg
                .message_text
                .split(" ")
                .map(|word| OsuMapSet::try_parse(word))
                .filter_map(|map| map)
                .collect();

            sender
                .send(OsuRelatedTwitchMessage {
                    sender: msg.sender.name,
                    message: msg.message_text,
                    maps,
                })
                .ok();
        }
    }
}
