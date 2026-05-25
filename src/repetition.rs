use crate::data_provider::voice::get_voice;
use crate::lang::{CardSet, CardStatistics, WordData, WordOpenMode};
use crate::repetitions::CardSetSettings;
use crate::Page::PreviousPage;
use crate::{AppState, KeyPressedPage, NavigatedPage, Page, RootMessage, DEFAULT_SPACING};
use iced::alignment::Horizontal::Center;
use iced::keyboard::key::Physical::Code;
use iced::widget::{button, column, container, row, rule, space, text, Column};
use iced::{alignment, keyboard, Element, Fill, Left, Task};
use rodio::MixerDeviceSink;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;

pub struct RepetitionState {
    pub settings: CardSetSettings,
    pub set: CardSet,
    current_word: WordData,
    current_statistic: CardStatistics,
    open: bool,
    can_play: bool,
    sink: Arc<MixerDeviceSink>,
    opened: HashSet<u32>,
}

impl NavigatedPage<RepetitionMessage> for RepetitionState {
    fn navigate(&self, message: &RepetitionMessage) -> Option<Page> {
        if let RepetitionMessage::Back = message {
            Some(PreviousPage)
        } else {
            None
        }
    }
}

impl RepetitionState {
    pub(crate) fn new(set: CardSetSettings, state: Arc<Mutex<AppState>>) -> RepetitionState {
        let mut card_set = CardSet::new(&set, state.clone());
        let (word, stat)  = card_set.next();
        let sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();

        RepetitionState {
            settings: set,
            set: card_set,
            current_word: word,
            current_statistic: stat,
            open: false,
            can_play: true,
            sink: Arc::new(sink_handle),
            opened: HashSet::new(),
        }
    }
}

impl RepetitionState {
    pub fn update(&mut self, message: RepetitionMessage) -> Task<RootMessage> {
        match message {
            RepetitionMessage::Back => {}
            RepetitionMessage::Next => return self.next(),
            RepetitionMessage::Answer(m) => return self.answer(m),
            RepetitionMessage::Play => {
                if !self.can_play {
                    return Task::none();
                }

                self.can_play = false;
                let value = self.current_word.key.clone();
                if self.settings.require_speech() {
                    return Task::perform(play_sound(self.sink.clone(), value), |_| {
                        RootMessage::Repetition(RepetitionMessage::PlayFinished)
                    });
                }
            }
            RepetitionMessage::PlayFinished => {
                self.can_play = true;
            }
        }

        Task::none()
    }

    fn next(&mut self) -> Task<RootMessage> {
        if self.open {
            self.answer(WordOpenMode::None)
        } else {
            self.open = true;
            Task::none()
        }
    }

    fn answer(&mut self, mode: WordOpenMode) -> Task<RootMessage> {
        if !self.open {
            return Task::none();
        }

        self.set.open(mode);
        self.open = false;
        self.opened.insert(self.current_word.id);
        let next = self.set.next();
        self.current_word = next.0;
        self.current_statistic = next.1;

        if self.settings.require_speech() {
            return Task::perform(
                play_sound(self.sink.clone(), self.current_word.key.clone()),
                |_| RootMessage::Repetition(RepetitionMessage::PlayFinished),
            );
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, RepetitionMessage> {
        container(
            iced::widget::column![
                button("Назад").on_press(RepetitionMessage::Back),
                column![
                    container(self.draw_forward())
                        .width(Fill)
                        .height(Fill)
                        .align_x(Center)
                        .align_y(alignment::Vertical::Center),
                    rule::horizontal(2),
                    container(self.draw_backward())
                        .width(Fill)
                        .height(Fill)
                        .align_x(Center)
                        .align_y(alignment::Vertical::Center),
                    container(self.answer_bar())
                        .width(Fill)
                        .align_x(Center)
                        .height(60),
                    text!(
                        "Затронуто слов {}, {}%",
                        self.opened.len(),
                        (self.opened.len() as f32 / self.set.len() as f32 * 10000.0).round()
                            / 100.0
                    )
                ]
                .height(Fill)
                .width(Fill)
            ]
            .align_x(Left)
            .width(Fill),
        )
        .center_x(Fill)
        .padding(10)
        .into()
    }

    fn draw_forward(&self) -> Element<'_, RepetitionMessage> {
        self.draw_card_view(self.settings.forward.as_str())

    }

    fn draw_backward(&self) -> Element<'_, RepetitionMessage> {
        if !self.open {
            return space().into();
        }


        self.draw_card_view(self.settings.backward.as_str())
    }

    fn draw_card_view(&self, properties: &str) -> Element<'_, RepetitionMessage> {
        let word = &self.current_word;

        let mut col = Column::new();

        for view_type in properties.split(" ") {
            col = col.push( match view_type {
                "key" => self.draw_key(word),
                "value" => self.draw_value(word),
                "speech" => self.draw_voice(),
                "reading" => self.draw_reading(word),
                "context" => self.draw_context(word),
                _ => space().into(),
            })
        }

        col.spacing(DEFAULT_SPACING).align_x(Center).into()
    }

    fn answer_bar(&self) -> Element<'_, RepetitionMessage> {
        if !self.open {
            return space().into();
        }

        column![
            text!("{} очков", self.current_statistic.calculated_score().round() as i32),
            row![
                button("Не получилось").on_press(RepetitionMessage::Answer(WordOpenMode::None)),
                button("Трудно").on_press(RepetitionMessage::Answer(WordOpenMode::Hard)),
                button("Нормально").on_press(RepetitionMessage::Answer(WordOpenMode::Ok)),
                button("Легко").on_press(RepetitionMessage::Answer(WordOpenMode::Easy)),
            ]
            .spacing(DEFAULT_SPACING)
        ]
        .align_x(Center)
        .spacing(DEFAULT_SPACING)
        .into()
    }

    fn draw_key(&self, word: &WordData) -> Element<'_, RepetitionMessage> {
        text!("{}", word.key).size(36).into()
    }
    fn draw_value(&self, word: &WordData) -> Element<'_, RepetitionMessage> {
        text!("{}", word.value).size(24).into()
    }

    fn draw_voice(&self) -> Element<'_, RepetitionMessage> {
        button("Воспроизвести")
            .on_press(RepetitionMessage::Play)
            .into()
    }

    fn draw_reading(&self, word: &WordData) -> Element<'_, RepetitionMessage> {
        match word.additional.get("reading") {
            None => space().into(),
            Some(reading) => text!("{}", reading).size(24).into(),
        }
    }
    fn draw_context(&self, word: &WordData) -> Element<'_, RepetitionMessage> {
        match word.additional.get("context") {
            None => space().into(),
            Some(context) => text!("{}", context).size(24).into(),
        }
    }
}

impl KeyPressedPage for RepetitionState {
    fn press(&mut self, message: &keyboard::Event) -> Task<RootMessage> {
        if let keyboard::Event::KeyPressed {
            key: _,
            modified_key: _,
            physical_key: pk,
            location: _,
            modifiers: _,
            text: _,
            repeat: _,
        } = message
        {
            if let Code(code) = pk {
                return match code {
                    keyboard::key::Code::Space => self.next(),
                    keyboard::key::Code::Digit1 => self.answer(WordOpenMode::None),
                    keyboard::key::Code::Digit2 => self.answer(WordOpenMode::Hard),
                    keyboard::key::Code::Digit3 => self.answer(WordOpenMode::Ok),
                    keyboard::key::Code::Digit4 => self.answer(WordOpenMode::Easy),
                    _ => Task::none(),
                };
            }
        }
        Task::none()
    }
}

#[derive(Clone)]
pub enum RepetitionMessage {
    Next,
    Back,
    Answer(WordOpenMode),
    Play,
    PlayFinished,
}

async fn play_sound(sink: Arc<MixerDeviceSink>, text: String) {
    let data = get_voice(text.as_str()).await;
    spawn_blocking(move || {
        rodio::play(&sink.mixer(), data).unwrap().sleep_until_end();
    })
    .await
    .unwrap();
}
