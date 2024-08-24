use std::error::Error;

use bytemuck::PodCastError;
use crossbeam_channel::SendError;

#[derive(Debug)]
pub enum BindGroupBuilderError {
    NoBufferFound(String),
    NoReadbackBufferFound(String),
    NoSenderFound(String),
    NoReceiverFound(String),
    NoBindGroupFound(String),
    NoLayoutFound(String),
    NoPipelineFound(String),
    ReceiverDistonnected,
    CastFailed(PodCastError),
    SendFailed(SendError<Vec<u8>>),
}
impl std::fmt::Display for BindGroupBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoBufferFound(name) => write!(f, "No buffer found with name {name}"),
            Self::NoReadbackBufferFound(name) => {
                write!(f, "No readback buffer found with name {name}")
            }
            Self::NoSenderFound(name) => write!(f, "No sender found with name {name}"),
            Self::NoReceiverFound(name) => write!(f, "No receiver found with name {name}"),
            Self::NoBindGroupFound(name) => write!(f, "No bind group found with name {name}"),
            Self::NoLayoutFound(name) => write!(f, "No layout found with name {name}"),
            Self::NoPipelineFound(name) => write!(f, "No pipeline found with name {name}"),
            Self::ReceiverDistonnected => write!(f, "Receiver is disconnected"),
            Self::CastFailed(err) => write!(f, "Cast failed: {err}"),
            Self::SendFailed(err) => write!(f, "Send failed: {err}"),
        }
    }
}
impl BindGroupBuilderError {
    pub fn no_buffer_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoBufferFound(format!("{name}"))
    }
    pub fn no_readback_buffer_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoReadbackBufferFound(format!("{name}"))
    }
    pub fn no_sender_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoSenderFound(format!("{name}"))
    }
    pub fn no_receiver_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoReceiverFound(format!("{name}"))
    }
    pub fn no_bind_group_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoBindGroupFound(format!("{name}"))
    }
    pub fn no_layout_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoLayoutFound(format!("{name}"))
    }
    pub fn no_pipeline_found<K: std::fmt::Display>(name: K) -> Self {
        Self::NoPipelineFound(format!("{name}"))
    }
}

impl Error for BindGroupBuilderError {}
