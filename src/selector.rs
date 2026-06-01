use std::sync::{Arc, Mutex};
use crate::dictionary::DictionaryState;
use crate::lang::{KanaSet, KanaType};
use crate::randomizer::randomizer::RandomizerState;
use crate::repetitions::RepetitionsState;
use crate::selector::SelectorMessage::ChangeMode;
use crate::writing::WritingState;
use crate::Page::{Quiz, Writing};
use crate::{AppState, NavigatedPage, Page, QuizState, RootMessage, DEFAULT_SPACING};
use iced::widget::*;
use iced::{alignment, Element, Task};
use crate::sync::SyncState;

pub struct SelectorState {
    pub set: KanaSet,
    is_writing: bool,
    state: Arc<Mutex<AppState>>
}

#[derive(Debug, Clone)]
pub enum SelectorMessage {
    Change,
    Goto,
    Check(usize, bool),
    ChangeMode(bool),
    ToDictionary,
    ToRandomize,
    ToRepetitions,
    ToSync,
}

impl NavigatedPage<SelectorMessage> for SelectorState {
    fn navigate(&self, message: &SelectorMessage) -> Option<Page> {
        if let SelectorMessage::Goto = message {
            return if self.is_writing {
                let writing = WritingState::new(&self.set);
                Some(Writing(writing))
            } else {
                let mut quiz = QuizState::new();
                quiz.set = self.set.clone();
                Some(Quiz(quiz))
            };
        }
        if let SelectorMessage::ToDictionary = message {
            return Some(Page::Dictionary(DictionaryState::new(self.state.clone())));
        }
        if let SelectorMessage::ToRandomize = message {
            return Some(Page::Randomizer(RandomizerState::default()));
        }
        if let SelectorMessage::ToRepetitions = message {
            return Some(Page::Repetitions(RepetitionsState::new(self.state.clone())));
        }
        if let SelectorMessage::ToSync = message {
            return Some(Page::Sync(SyncState::new(self.state.clone())));
        }
        None
    }
}

impl SelectorState {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self{
            set: Default::default(),
            is_writing: false,
            state,
        }
    }
    pub fn update(&mut self, message: SelectorMessage) -> Task<RootMessage> {
        match message {
            SelectorMessage::Change => match self.set.chars_type {
                KanaType::Katakana => self.set = KanaSet::hiragana(),
                KanaType::Hiragana => self.set = KanaSet::katakana(),
            },
            SelectorMessage::Check(i, b) => self.set.include_map[i] = b,
            ChangeMode(b) => self.is_writing = b,
            _ => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, SelectorMessage> {
        container(
            iced::widget::column![
                row![
                    button("Переключить азбуки").on_press(SelectorMessage::Change),
                    button("Словарь").on_press(SelectorMessage::ToDictionary),
                    button("Рандомайзер").on_press(SelectorMessage::ToRandomize),
                    button("Повторение").on_press(SelectorMessage::ToRepetitions),
                    button("Синхронизация").on_press(SelectorMessage::ToSync)
                ]
                .spacing(DEFAULT_SPACING),
                self.rows_selector(),
                toggler(self.is_writing)
                    .label("Режим письма")
                    .on_toggle(ChangeMode),
                button("К тесту").on_press(SelectorMessage::Goto),
            ]
            .spacing(DEFAULT_SPACING),
        )
        .padding(10)
        .into()
    }

    fn rows_selector(&self) -> Element<'_, SelectorMessage> {
        let mut row = Row::new();

        for i in 0..self.set.dictionary.len() {
            let setup_checked = move |b: bool| -> SelectorMessage { SelectorMessage::Check(i, b) };

            let mut chars_column: Column<'_, _> = Column::new();
            chars_column =
                chars_column.push(checkbox(self.set.include_map[i]).on_toggle(setup_checked));

            for v in &self.set.dictionary[i] {
                chars_column = chars_column.push(
                    container(text!("{}", v.0.clone().to_uppercase()).size(36))
                        .padding(20)
                        .style(container::rounded_box),
                );
            }

            row = row.push(
                chars_column
                    .spacing(DEFAULT_SPACING)
                    .align_x(alignment::Horizontal::Center),
            );
        }

        row.spacing(DEFAULT_SPACING).into()
    }
}
