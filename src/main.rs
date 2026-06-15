#![windows_subsystem = "windows"]
mod data_provider;
mod dictionary;
mod dictionary_test;
mod lang;
mod quiz;
mod randomizer;
mod repetition;
mod repetitions;
mod selector;
mod sync;
mod word;
mod writing;

use crate::data_provider::card_sets::load_sets;
use crate::data_provider::settings::get_setting;
use crate::data_provider::words::{ load_word_groups, load_words};
use crate::dictionary::{app_data_dir, DictionaryMessage, DictionaryState};
use crate::dictionary_test::{DictionaryQuizMessage, DictionaryQuizState};
use crate::lang::{WordData, WordGroup};
use crate::quiz::*;
use crate::randomizer::randomizer::{RandomizerMessage, RandomizerState};
use crate::repetition::{RepetitionMessage, RepetitionState};
use crate::repetitions::{CardSetSettings, RepetitionsMessage, RepetitionsState};
use crate::selector::*;
use crate::sync::{SyncMessage, SyncState};
use crate::word::{WordMessage, WordState};
use crate::writing::{WritingMessage, WritingState};
use crate::Page::{
    Dictionary, DictionaryQuiz, Quiz, Randomizer, Repetition, Repetitions, Selector, Sync, Word,
    Writing,
};
use crate::RootMessage::Keyboard;
use iced::keyboard::Event;
use iced::widget::text;
use iced::{keyboard, Element, Subscription};
use iced::{Font, Task};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use crate::data_provider::sqlite::create_db;

const DEFAULT_SPACING: f32 = 10.0;

fn main() -> iced::Result {
    iced::application(ScreenState::boot, ScreenState::update, ScreenState::view)
        .subscription(subscription)
        .title("Kana learn app")
        .font(include_bytes!("../noto.ttf"))
        .default_font(Font::with_name("Noto Sans JP"))
        .run()
}

fn subscription(_state: &ScreenState) -> Subscription<RootMessage> {
    keyboard::listen().map(|e| Keyboard(e))
}

#[derive(Clone)]
pub enum RootMessage {
    Selector(SelectorMessage),
    Quiz(QuizMessage),
    Writing(WritingMessage),
    Dictionary(DictionaryMessage),
    DictionaryQuiz(DictionaryQuizMessage),
    Randomizer(RandomizerMessage),
    Repetitions(RepetitionsMessage),
    Repetition(RepetitionMessage),
    Word(WordMessage),
    Sync(SyncMessage),
    Keyboard(Event),
}

pub enum Page {
    Selector(SelectorState),
    Quiz(QuizState),
    Writing(WritingState),
    Dictionary(DictionaryState),
    DictionaryQuiz(DictionaryQuizState),
    Randomizer(RandomizerState),
    Repetitions(RepetitionsState),
    Repetition(RepetitionState),
    Word(WordState),
    Sync(SyncState),
    PreviousPage,
}

pub struct ScreenState {
    stack: Vec<Page>,
}

pub struct AppState {
    pub dictionary: Vec<WordData>,
    pub card_sets: Vec<CardSetSettings>,
    pub word_groups: Vec<WordGroup>,
    pub connection: Connection,
    pub sync_data: AppSettings,
}

impl Default for ScreenState {
    fn default() -> Self {
        let path = app_data_dir();
        let db_file = path.join("data.db");
        let connection = Connection::open(db_file).unwrap();
        connection.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        let list = load_words(&connection);
        let sets = load_sets(&connection);
        let groups = load_word_groups(&connection);
        let setting = load_settings(&connection);

        let state = Arc::new(Mutex::new(AppState {
            dictionary: list,
            card_sets: sets,
            connection,
            word_groups: groups,
            sync_data: setting,
        }));
        ScreenState {
            stack: vec![Selector(SelectorState::new(state.clone()))],
        }
    }
}

fn load_settings(connection: &Connection) -> AppSettings {
    let key = get_setting("SYNC_KEY".to_string(), connection);
    AppSettings { key }
}

impl ScreenState {
    pub fn boot() -> (ScreenState, Task<RootMessage>) {
        create_db();
        (ScreenState::default(), Task::none())
    }
    pub fn update(&mut self, message: RootMessage) -> Task<RootMessage> {
        if let Keyboard(e) = message {
            return self.handle_keyboard(e);
        }

        state_update!(
            message,
            self.stack,
            Selector,
            Quiz,
            Writing,
            Dictionary,
            DictionaryQuiz,
            Randomizer,
            Repetitions,
            Repetition,
            Word,
            Sync
        );
        Task::none()
    }
    pub fn view(&self) -> Element<'_, RootMessage> {
        view_navigation!(
            self.stack,
            Quiz,
            Selector,
            Writing,
            Dictionary,
            DictionaryQuiz,
            Randomizer,
            Repetitions,
            Repetition,
            Word,
            Sync
        )
    }

    fn handle_keyboard(&mut self, message: Event) -> Task<RootMessage> {
        let page = self.stack.last_mut().unwrap();
        match page {
            Repetition(page) => page.press(&message),
            _ => Task::none(),
        }
    }
}

pub struct AppSettings {
    key: Option<String>,
}

#[macro_export]
macro_rules! view_navigation {
    ($stack:expr, $($e:ident), *) => {
        match &$stack.last().unwrap() {
            $(
            $e(s) => s.view().map(RootMessage::$e),
            )*
            _ => text!("").into(),
        }
    }
}

#[macro_export]
macro_rules! state_update {
    ($message:expr, $stack:expr, $($e:ident), *) => {
        match $message {
            $(
            RootMessage::$e(msg) => {
                if let $e(s) = $stack.last_mut().unwrap() {
                    message_navigation!(msg, $stack, s)
                }
            }
            )*
            _ => {}
        }
    }
}

#[macro_export]
macro_rules! message_navigation {
    ($msg:expr, $stack:expr, $state:expr) => {
        if let Some(new_page) = $state.navigate(&$msg) {
            if let Page::PreviousPage = new_page {
                $stack.pop();
                return Task::none();
            }
            $stack.push(new_page);
        } else {
            return $state.update($msg);
        }
    };
}

trait NavigatedPage<T> {
    fn navigate(&self, message: &T) -> Option<Page>;
}

pub trait KeyPressedPage {
    fn press(&mut self, message: &Event) -> Task<RootMessage>;
}
