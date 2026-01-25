pub use crate::{
    bot::MerilBot,
    core::api::NapcatApi,
    core::event::EventManager,
    utils::parser::{
        message_parser::Message,
        request_parser::{GroupMessage, NapcatRequestData, PrivateMessage},
    },
};
