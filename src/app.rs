use crate::event::{AppEvent, Event, EventHandler};

use gst::prelude::*;

use gstreamer::{self as gst, ClockTime, SeekFlags};
use gstreamer_pbutils::Discoverer;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    text::ToText,
};
use rfd::FileHandle;

use std::time::{Duration, Instant};
use std::{panic, thread};
use std::{path::Path, sync::mpsc};
use url::Url;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub counter: u64,
    pub events: EventHandler,
    pub name: String,
    pub pause: bool,
    pub player_tx: Option<mpsc::Sender<PlayerCommand>>,
    pub video_path: String,
    pub change_vid: String,
    pub player_var_tx: Option<mpsc::Sender<PlayerVars>>,
    pub video_duration: String,
    pub player_info_tx: Option<mpsc::Receiver<PlayerSend>>,
    pub video_time: String,
    pub video_opened: bool,
    pub last_receive: Instant,
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Change,
    Seek,
    Get,
}

#[derive(Debug, Clone)]
pub enum PlayerVars {
    SeekTime(u64),
    VideoFile(Url),
    // video_file: String,
}

#[derive(Debug, Clone)]
pub enum PlayerSend {
    CurrentTime(ClockTime),
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            name: "Koushik".to_string(),
            pause: false,
            player_tx: None,
            player_var_tx: None,
            video_path: "/home/koushikk/Downloads/foden.mkv".to_string(),
            change_vid: "/home/koushikk/Downloads/SHOWS/OWAIMONO/[Commie] Owarimonogatari [BD 720p AAC]/[Commie] Owarimonogatari - 03 [BD 720p AAC] [371E3589].mkv".to_string(),
            video_duration: String::new(),
            player_info_tx: None,
            video_time: String::new(),
            video_opened: false,
            last_receive: Instant::now(),




        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::ChangeName => self.change_name(),
                    AppEvent::ChangeVid => self.change().await,
                    AppEvent::Quit => self.quit(),
                    AppEvent::ChangeTime => self.seeker(),
                    AppEvent::Receive => self.receive(),
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            KeyCode::Up => self.events.send(AppEvent::ChangeName),
            KeyCode::Char('k') => self.events.send(AppEvent::ChangeVid),
            KeyCode::Char('t') => self.events.send(AppEvent::ChangeTime),
            KeyCode::Char('r') => self.events.send(AppEvent::Receive),

            _ => {}
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        if self.video_opened && self.last_receive.elapsed() > Duration::from_millis(100) {
            if let Some(tx) = &self.player_tx {
                let _ = tx.send(PlayerCommand::Get);
            }

            if let Some(rx) = &self.player_info_tx {
                if let Ok(PlayerSend::CurrentTime(time)) = rx.try_recv() {
                    self.video_time = time.to_string();
                    self.last_receive = Instant::now();
                }
            }
        }
    }

    pub fn change_name(&mut self) {
        self.name = "lebron".to_string();
        self.pause = true;

        match &self.player_tx {
            Some(tx) => {
                let _ = tx.send(PlayerCommand::Pause);
            }
            None => {
                println!("nothing happens");
            }
        }
    }

    pub fn tutorial_main(&mut self, path_string: String) {
        let (tx, rx) = mpsc::channel::<PlayerCommand>();
        self.player_tx = Some(tx.clone());

        let (sendvar, recvvar) = mpsc::channel::<PlayerVars>();
        self.player_var_tx = Some(sendvar.clone());

        let (playerinfosend, player_send_rx) = mpsc::channel::<PlayerSend>();
        self.player_info_tx = Some(player_send_rx);

        thread::spawn(move || {
            gst::init().unwrap();

            let path = Path::new(path_string.as_str());

            let url = match Url::from_file_path(path) {
                Ok(url) => url,
                Err(e) => {
                    println!("error creaing url {:?}", e);
                    panic!("failure url");
                }
            };

            let playbin = gst::ElementFactory::make("playbin").build().unwrap();
            playbin.set_property("uri", url.to_string());
            playbin.set_state(gst::State::Playing).unwrap();

            loop {
                match rx.recv() {
                    Ok(PlayerCommand::Play) => {
                        playbin
                            .set_state(gst::State::Playing)
                            .expect("Unable to play");
                    }
                    Ok(PlayerCommand::Change) => match recvvar.recv() {
                        Ok(PlayerVars::SeekTime(_)) => {}
                        Ok(PlayerVars::VideoFile(url5)) => {
                            println!("file : {:?}", url5);
                            playbin.set_state(gst::State::Null).unwrap();
                            playbin.set_property("uri", url5.to_string());

                            playbin.set_state(gst::State::Playing).unwrap();
                        }
                        Err(e) => {
                            println!("error {:?}", e);
                        }
                    },
                    Ok(PlayerCommand::Pause) => {
                        playbin
                            .set_state(gst::State::Paused)
                            .expect("Unable to pause");
                    }
                    Ok(PlayerCommand::Stop) | Err(_) => {
                        playbin.set_state(gst::State::Null).expect("Unable to stop");
                        break;
                    }
                    Ok(PlayerCommand::Get) => {
                        if let Some(position) = playbin.query_position::<gst::ClockTime>() {
                            let _ = playerinfosend.send(PlayerSend::CurrentTime(position));
                        } else {
                            //println!("fuark")
                        }
                    }
                    Ok(PlayerCommand::Seek) => {
                        let mut pos = gst::ClockTime::from_seconds(30);
                        match recvvar.recv() {
                            Ok(PlayerVars::SeekTime(time)) => {
                                pos = gst::ClockTime::from_seconds(time);
                            }
                            Err(e) => {
                                println!("error receing playervar {:?}", e);
                            }
                            Ok(PlayerVars::VideoFile(_)) => {}
                        }
                        playbin
                            .seek_simple(SeekFlags::FLUSH | SeekFlags::KEY_UNIT, pos)
                            .unwrap();
                    }
                }
            }
        });
    }
    pub fn quit(&mut self) {
        self.running = false;

        match &self.player_tx {
            Some(tx) => {
                let _ = tx.send(PlayerCommand::Stop);
            }
            None => {
                println!("ntohing happens");
            }
        }

        self.decrement_counter();
    }

    pub fn receive(&mut self) {
        // send the get
        if let Some(tx) = &self.player_tx {
            let _ = tx.send(PlayerCommand::Get);
        }
    }

    pub async fn change(&mut self) {
        let video_from_rfd = self.open_file().await.unwrap();
        let path_new = video_from_rfd.path().to_string_lossy().to_string();
        let discover = Discoverer::new(ClockTime::from_seconds(5)).expect("failed");
        let url = Url::from_file_path(path_new.clone()).unwrap();
        let info = discover.discover_uri(url.as_str()).unwrap();
        let duration = info.duration().unwrap().to_text().to_string();

        self.video_duration = duration;

        match &self.player_tx {
            Some(tx) => {
                match &self.player_var_tx {
                    Some(tx2) => {
                        let path3 = Path::new(path_new.as_str());

                        let url3 = match Url::from_file_path(path3) {
                            Ok(url3) => url3,
                            Err(e) => {
                                println!("error creaing url {:?}", e);
                                return;
                            }
                        };
                        let _ = tx2.send(PlayerVars::VideoFile(url3));
                    }
                    None => {
                        println!("failted to change");
                    }
                }
                let _ = tx.send(PlayerCommand::Change);
            }
            None => {
                println!("nothing changed");
            }
        }
    }

    pub fn seeker(&mut self) {
        match &self.player_tx {
            Some(tx) => {
                match &self.player_var_tx {
                    Some(tx2) => {
                        let _ = tx2.send(PlayerVars::SeekTime(self.counter));
                    }
                    None => {
                        println!("fukced up the player var thing");
                    }
                }
                let _ = tx.send(PlayerCommand::Seek);
            }
            None => {
                println!("nothing happnes while seeking");
            }
        }
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);

        match &self.player_tx {
            Some(tx) => {
                let _ = tx.send(PlayerCommand::Play);
            }
            None => {
                println!("nothing bluds");
            }
        }
    }

    pub fn decrement_counter(&mut self) {
        if self.player_tx.is_none() {
            self.tutorial_main(self.video_path.clone());
        }
        self.video_opened = true;

        self.counter = self.counter.saturating_sub(1);
    }

    pub async fn open_file(&mut self) -> Option<FileHandle> {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .set_title("Select a file")
            .add_filter("video files", &["mp4", "avi", "mkv", "mov"])
            .pick_file()
            .await
        {
            println!("Selected file {:?}", handle.path());
            Some(handle)
        } else {
            println!("no file selected");
            None
        }
    }
}
