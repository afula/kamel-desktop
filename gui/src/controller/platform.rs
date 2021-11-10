use std::thread::{self, JoinHandle};

use crate::controller::command;
use druid::{
    im::Vector,
    widget::{prelude::*, Controller},
    Code, ExtEventSink, InternalLifeCycle, KbKey, WindowHandle,
};
use signal::AppData;

pub struct PlatformController {
    // sender: Option<Sender<bool>>,
    thread: Option<JoinHandle<()>>,
}

impl PlatformController {
    pub fn new() -> Self {
        Self {
            // sender: None,
            thread: None,
        }
    }

    /*    fn open_audio_output_and_start_threads(
        &mut self,
        event_sink: ExtEventSink,
        widget_id: WidgetId,
    ) {
    }

    fn service_events(mut player: Player, event_sink: ExtEventSink, widget_id: WidgetId) {
        for event in player.event_receiver() {
            // Forward events that affect the UI state to the UI thread.
            println!("service_events: {:?}",event);
            match &event {
                PlayerEvent::Loading { item } => {
                    let item: TrackId = TrackId::from_str(item).unwrap();
                    event_sink
                        .submit_command(command::PLAYBACK_LOADING, item, widget_id)
                        .unwrap();
                }
                PlayerEvent::Playing { path, position } => {
                    let item: TrackId = TrackId::from_str(path).unwrap();
                    let progress = position.to_owned();
                    event_sink
                        .submit_command(command::PLAYBACK_PLAYING, (item, progress), widget_id)
                        .unwrap();
                }
                PlayerEvent::Pausing { .. } => {
                    event_sink
                        .submit_command(command::PLAYBACK_PAUSING, (), widget_id)
                        .unwrap();
                }
                PlayerEvent::Resuming { .. } => {
                    println!("PlayerEvent::Resuming");
                    event_sink
                        .submit_command(command::PLAYBACK_RESUMING, (), widget_id)
                        .unwrap();
                }
                PlayerEvent::Position { position, .. } => {
                    let progress = position.to_owned();
                    event_sink
                        .submit_command(command::PLAYBACK_PROGRESS, progress, widget_id)
                        .unwrap();
                }
                PlayerEvent::Blocked { .. } => {
                    event_sink
                        .submit_command(command::PLAYBACK_BLOCKED, (), widget_id)
                        .unwrap();
                }
                PlayerEvent::Stopped => {
                    event_sink
                        .submit_command(command::PLAYBACK_STOPPED, (), widget_id)
                        .unwrap();
                }
                PlayerEvent::Report { duration, position } => {
                    println!("controller report");
                    let duration = duration.to_owned();
                    let position = position.to_owned();
                    event_sink
                        .submit_command(command::PLAYBACK_REPORT, (duration,position), widget_id)
                        .unwrap();
                }
                _ => {}
            }

            // Let the player react to its internal events.
            player.handle(event);
        }
    }

    fn send(&mut self, event: PlayerEvent) {
        self.sender.as_mut().unwrap().send(event).unwrap();
    }

    fn play(&mut self, item: &QueuedTrack) {
        self.send(PlayerEvent::Command(PlayerCommand::LoadAndPlay {
            item: QueuedTrack{
                track: item.track.clone()
            }
        }));
    }

    fn pause(&mut self) {
        self.send(PlayerEvent::Command(PlayerCommand::Pause));
    }

    fn resume(&mut self) {
        self.send(PlayerEvent::Command(PlayerCommand::Resume));
    }

    fn pause_or_resume(&mut self) {
        self.send(PlayerEvent::Command(PlayerCommand::PauseOrResume));
    }

    fn previous(&mut self) {
        self.send(PlayerEvent::Command(PlayerCommand::Previous));
    }

    fn next(&mut self) {
        self.send(PlayerEvent::Command(PlayerCommand::Next));
    }

    fn stop(&mut self) {
        self.send(PlayerEvent::Command(PlayerCommand::Stop));
    }

    fn seek(&mut self, position: Duration) {
        self.send(PlayerEvent::Command(PlayerCommand::Seek { position }));
    }

    fn set_volume(&mut self, volume: f64) {
        self.send(PlayerEvent::Command(PlayerCommand::SetVolume { volume }));
    }*/
}

impl<W> Controller<AppData, W> for PlatformController
where
    W: Widget<AppData>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(command::SET_CURRENT_PLATFORM) => {
                let platform = cmd.get_unchecked(command::SET_CURRENT_PLATFORM);

                data.current_platform.replace(platform.to_owned());
                // data.set_queue_behavior(behavior.to_owned());
                // self.set_queue_behavior(behavior.to_owned());
                ctx.set_handled();
            }

            //
            Event::KeyDown(key) if key.code == Code::Space => {
                // self.pause_or_resume();
                ctx.set_handled();
            }
            Event::KeyDown(key) if key.code == Code::ArrowRight => {
                // self.next();
                ctx.set_handled();
            }
            Event::KeyDown(key) if key.code == Code::ArrowLeft => {
                // self.previous();
                ctx.set_handled();
            }
            Event::KeyDown(key) if key.key == KbKey::Character("+".to_string()) => {
                // data.playback.volume = (data.playback.volume + 0.1).min(1.0);
                ctx.set_handled();
            }
            Event::KeyDown(key) if key.key == KbKey::Character("-".to_string()) => {
                // data.playback.volume = (data.playback.volume - 0.1).max(0.0);
                ctx.set_handled();
            }
            //
            _ => child.event(ctx, event, data, env),
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                /*                println!("playback controller WidgetAdded");
                self.open_audio_output_and_start_threads(
                    ctx.get_external_handle(),
                    ctx.widget_id(),
                    ctx.window(),
                );
                self.set_volume(data.playback.volume);*/

                /*                // Request focus so we can receive keyboard events.
                ctx.submit_command(command::SET_FOCUS.to(ctx.widget_id()));*/
            }
            LifeCycle::Internal(InternalLifeCycle::RouteFocusChanged { new: None, .. }) => {
                // Druid doesn't have any "ambient focus" concept, so we catch the situation
                // when the focus is being lost and sign up to get focused ourselves.
                // ctx.submit_command(command::SET_FOCUS.to(ctx.widget_id()));
            }
            _ => {}
        }
        child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &AppData,
        data: &AppData,
        env: &Env,
    ) {
        /*        if !old_data.playback.volume.same(&data.playback.volume) {
            self.set_volume(data.playback.volume);
        }*/
        child.update(ctx, old_data, data, env);
    }
}
