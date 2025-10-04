use crate::event::{AppEvent, Event, EventHandler};
use gst::prelude::*;
use gstreamer::{self as gst, SeekFlags};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use std::{panic, path::PathBuf, thread};
use std::{
    path::{self, Path},
    sync::{Arc, Mutex, mpsc},
};
use url::Url;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub counter: u8,
    pub events: EventHandler,
    pub name: String,
    pub pause: bool,
    pub player_tx: Option<mpsc::Sender<PlayerCommand>>,
    pub video_path: String,
    pub change_vid: String,
    pub player_var_tx: Option<mpsc::Sender<PlayerVars>>,
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Change,
    Seek,
}

#[derive(Debug, Clone)]
pub enum PlayerVars {
    SeekTime(u64),
    VideoFile(Url),
    // video_file: String,
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
                    AppEvent::ChangeVid => self.change(),
                    AppEvent::Quit => self.quit(),
                    AppEvent::ChangeTime => self.seeker(),
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

            _ => {}
        }
        Ok(())
    }

    pub fn tick(&self) {}

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

    pub fn tutorial_main(&mut self, path_string: String, second_path: String) {
        let (tx, rx) = mpsc::channel::<PlayerCommand>();
        self.player_tx = Some(tx.clone());

        let (sendvar, recvvar) = mpsc::channel::<PlayerVars>();
        self.player_var_tx = Some(sendvar.clone());

        thread::spawn(move || {
            gst::init().unwrap();

            //let uri = "https://gstreamer.freedesktop.org/data/media/sintel_trailer-480p.webm".to_string();
            // let path = Path::new("/home/koushikk/Downloads/foden.mkv");
            // self.video_path = path.clone().to_string_lossy().to_string();
            // let path: &Path = match Path::new("/home/koushikk/Downloads/foden.mkv") {
            //     path => path,
            // };
            // let path = Path::new(path_str);
            // let path = PathBuf::from(path_string);
            //
            let path = Path::new(path_string.as_str());

            let url = match Url::from_file_path(path) {
                Ok(url) => url,
                Err(e) => {
                    println!("error creaing url {:?}", e);
                    panic!("failure url");
                }
            };

            let path2 = Path::new(second_path.as_str());

            let url2 = match Url::from_file_path(path2) {
                Ok(url2) => url2,
                Err(e) => {
                    println!("error creaing url {:?}", e);
                    return;
                }
            };

            let playbin = gst::ElementFactory::make("playbin").build().unwrap();
            playbin.set_property("uri", url.to_string());
            playbin.set_state(gst::State::Playing).unwrap();

            // let pipeline = gst::parse::launch(&format!("playbin uri={url}")).unwrap();
            loop {
                match rx.recv() {
                    Ok(PlayerCommand::Play) => {
                        playbin
                            .set_state(gst::State::Playing)
                            .expect("Unable to play");
                    }
                    Ok(PlayerCommand::Change) => {
                        match recvvar.recv() {
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
                        }

                        // let path3 = Path::new(file.as_str());
                        //
                        // let mut url3 = Url::from_file_path(Path::new(".")).expect("bro wtf");
                        //
                        // url3 = match Url::from_file_path(path3) {
                        //     Ok(url3) => url3,
                        //     Err(e) => {
                        //         println!("error creaing url {:?}", e);
                        //         println!("{}", url3);
                        //         return;
                        //     }
                        // };

                        // playbin.set_state(gst::State::Null).unwrap();
                        // playbin.set_property("uri", url5.to_string());
                        // playbin.set_state(gst::State::Playing).unwrap();
                    }
                    Ok(PlayerCommand::Pause) => {
                        playbin
                            .set_state(gst::State::Paused)
                            .expect("Unable to pause");
                    }
                    Ok(PlayerCommand::Stop) | Err(_) => {
                        playbin.set_state(gst::State::Null).expect("Unable to stop");
                        break;
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

    pub fn change(&mut self) {
        // let path_new = "/home/koushikk/Downloads/ssstwitter.com_1752545017404.mp4".to_string();
        let path_new = self.change_vid.clone();

        // need to find a better way to send struct, because i dont want to send a whole struct i
        // just want to send a value but the way it is right now im receiving the whole struct
        // ok thats just fuckign enums
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
                        let _ = tx2.send(PlayerVars::SeekTime(self.counter.into()));
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
            self.tutorial_main(self.video_path.clone(), self.change_vid.clone());
        }

        self.counter = self.counter.saturating_sub(1);
    }
}
