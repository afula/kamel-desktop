#![windows_subsystem = "windows"]

use async_std::net::TcpStream;
use async_std::task::block_on;
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{Attribute, RichText, TextStorage};
use druid::widget::prelude::*;
use druid::widget::{Button, Controller, Flex, Label, LineBreaking, RadioGroup, RawLabel, Scroll};
use druid::{
    AppLauncher, Color, Data, FontFamily, FontStyle, FontWeight, Lens, LocalizedString,
    TextAlignment, Widget, WidgetExt, WindowDesc,
};
use futures::{AsyncReadExt, AsyncWriteExt};
use gui::states::OWNER;

fn main() {
    unimplemented!()
}
