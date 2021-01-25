use dbus::arg;
use dbus::blocking::Connection;
use dbus_crossroads::{Context, Crossroads};
use futures::channel::mpsc::Sender;
use std::thread;

use crate::app::AppAction;

mod generated;
use generated::OrgMprisMediaPlayer2Player as MprisPlayer;

struct SpotMprisPlayer(Sender<AppAction>);

impl MprisPlayer for SpotMprisPlayer {
    fn next(&mut self) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn previous(&mut self) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn pause(&mut self) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn play_pause(&mut self) -> Result<(), dbus::MethodErr> {
        self.0.clone().try_send(AppAction::TogglePlay).unwrap();
        Ok(())
    }

    fn stop(&mut self) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn play(&mut self) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn seek(&mut self, offset: i64) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn set_position(
        &mut self,
        track_id: dbus::Path<'static>,
        position: i64,
    ) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn open_uri(&mut self, uri: String) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn playback_status(&self) -> Result<String, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn loop_status(&self) -> Result<String, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn set_loop_status(&self, value: String) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn rate(&self) -> Result<f64, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn set_rate(&self, value: f64) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn shuffle(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }

    fn set_shuffle(&self, value: bool) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn metadata(&self) -> Result<arg::PropMap, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn volume(&self) -> Result<f64, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn set_volume(&self, value: f64) -> Result<(), dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn position(&self) -> Result<i64, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn minimum_rate(&self) -> Result<f64, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn maximum_rate(&self) -> Result<f64, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn can_go_next(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn can_go_previous(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn can_play(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn can_pause(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn can_seek(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
    fn can_control(&self) -> Result<bool, dbus::MethodErr> {
        Err(dbus::MethodErr::failed("unimplemented"))
    }
}

pub fn start_dbus_service(appaction_sender: Sender<AppAction>) {
    thread::spawn(move || {
        let c = Connection::new_session().unwrap();
        c.request_name("org.mpris.MediaPlayer2.Spot", true, false, false)
            .unwrap();

        let mut cr = Crossroads::new();
        let iface_token = generated::register_org_mpris_media_player2_player(&mut cr);
        cr.insert("/Player", &[iface_token], SpotMprisPlayer(appaction_sender));
        cr.serve(&c).unwrap();
    });
}
