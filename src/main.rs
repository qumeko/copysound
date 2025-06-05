/*
TODO:
1. Сделать возможность выбора пути к файлу в GUI.
2. Реализовать сохранение пути файла и громкости.  
*/


#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Cursor;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use inputbot::{KeybdKey::*,};
use rodio::{Decoder, OutputStream, Sink,};
use eframe::{egui, App, Frame, CreationContext};
use egui::Context;

const SOUND_BYTES: &[u8] = include_bytes!("sound.wav");

static SHARED_VOLUME: Lazy<Mutex<f32>> = Lazy::new(|| Mutex::new(0.5_f32));

struct MyApp {
    slider_value: f32,
    apply_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let initial_volume = SHARED_VOLUME.lock()
            .map(|guard| *guard)
            .unwrap_or_else(|e| {
                eprintln!("Failed to lock shared volume on init: {}", e);
                0.5_f32
            });
        Self {
            slider_value: initial_volume,
            apply_message: String::new(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ctrl + C Sound");
            ui.separator();

            ui.add(egui::Slider::new(&mut self.slider_value, 0.0..=1.0).text("Громкость"));
            ui.label(format!("Выбранная громкость: {:.0}%", self.slider_value * 100.0));

            let current_active_volume = SHARED_VOLUME.lock()
                .map(|guard| *guard)
                .unwrap_or_else(|e| {
                    eprintln!("Failed to lock shared volume for display: {}", e);
                    self.slider_value
                });
            ui.label(format!("Активная громкость для звука: {:.0}%", current_active_volume * 100.0));

            ui.add_space(10.0);

            if ui.button("Применить громкость").clicked() {
                match SHARED_VOLUME.lock() {
                    Ok(mut volume_lock) => {
                        *volume_lock = self.slider_value;
                        self.apply_message = format!("Громкость {:.0}% применена!", self.slider_value * 100.0);
                        println!("Volume set to: {}", self.slider_value);
                    }
                    Err(e) => {
                        self.apply_message = format!("Ошибка сохранения громкости: {}", e);
                        eprintln!("Failed to lock shared volume to set: {}", e);
                    }
                }
            }
            ui.label(&self.apply_message);
        });
    }
}

fn playsound() {
    let volume_to_play = SHARED_VOLUME.lock()
        .map(|guard| *guard)
        .unwrap_or_else(|e| {
            eprintln!("Failed to lock shared volume for playing sound: {}", e);
            0.5_f32
        });

    match OutputStream::try_default() {
        Ok((_stream, stream_handle)) => {
            let file_cursor = Cursor::new(SOUND_BYTES);
            match Decoder::new(file_cursor) {
                Ok(source) => {
                    match Sink::try_new(&stream_handle) {
                        Ok(sink) => {
                            sink.set_volume(volume_to_play);
                            sink.append(source);
                            sink.sleep_until_end();
                        }
                        Err(e) => println!("Error creating sink: {}", e),
                    }
                }
                Err(e) => println!("Error decoding sound: {}", e),
            }
        }
        Err(e) => println!("Error getting default output stream: {}", e),
    }
}

fn main() -> Result<(), eframe::Error> {
    std::thread::spawn(|| {
        CKey.bind(|| {
            if LControlKey.is_pressed() || RControlKey.is_pressed() {
                println!("Ctrl+C pressed, playing sound...");
                playsound();
            }
        });
        inputbot::handle_input_events();
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(320.0, 280.0)), 
        ..Default::default()
    };

    eframe::run_native(
        "Ctrl+C Sound Setter",
        options,
        Box::new(|_cc: &CreationContext<'_>| {
            Ok(Box::new(MyApp::default()) as Box<dyn App>)
        }),
    )
}