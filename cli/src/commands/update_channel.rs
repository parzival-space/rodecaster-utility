use std::time::Duration;
use clap::Parser;
use log::{error, info};
use crate::commands::{Command, CommandContext};
use crate::commands::dump::DumpCommand;

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
pub struct UpdateChannelArguments {
    #[clap(short, long)]
    pub channel: i8,

    #[clap(short, long, default_value_t = false)]
    pub mute: bool,

    #[clap(short, long, default_value_t = false)]
    pub solo: bool,
}

pub struct UpdateChannelCommand {}
impl UpdateChannelCommand {
    pub fn execute(context: CommandContext, arguments: UpdateChannelArguments) {
        println!("{}", arguments.channel);
    }
}