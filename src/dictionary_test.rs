use crate::dictionary::{split_with_coma};
use crate::quiz::Score;
use crate::Page::PreviousPage;
use crate::{RootMessage, DEFAULT_SPACING};
use crate::{NavigatedPage, Page};
use iced::border::Radius;
use iced::widget::container::Style;
use iced::widget::{button, container, row, space, text, text_input, Row};
use iced::Background::Color;
use iced::{alignment, Border, Element, Fill, Task, Theme};
use rand::prelude::SliceRandom;
use crate::lang::WordData;

#[derive(Debug, Clone)]
pub struct DictionaryQuizState {
    words: Vec<WordData>,
    current_set: Vec<WordData>,
    answer: String,
    view: String,
    correct: String,
    score: Score,
    is_help: bool,
    reverse: bool,
    laps: u32,
    no_typing: bool,
}
#[derive(Debug, Clone)]
pub enum DictionaryQuizMessage {
    Back,
    AnswerChanged(String),
    SubmitAnswer,
    Appeal,
}

impl NavigatedPage<DictionaryQuizMessage> for DictionaryQuizState {
    fn navigate(&self, message: &DictionaryQuizMessage) -> Option<Page> {
        match message {
            DictionaryQuizMessage::Back => Some(PreviousPage),
            _ => None,
        }
    }
}

impl DictionaryQuizState {
    pub fn new(
        words: Vec<WordData>,
        reverse: bool,
        no_typing: bool,
    ) -> DictionaryQuizState {
        DictionaryQuizState {
            words,
            current_set: Vec::new(),
            answer: "".to_string(),
            view: "---".to_string(),
            correct: "".to_string(),
            score: Default::default(),
            is_help: false,
            reverse,
            laps: 0,
            no_typing,
        }
    }

    pub fn update(&mut self, message: DictionaryQuizMessage) -> Task<RootMessage> {
        match message {
            DictionaryQuizMessage::Back => {}
            DictionaryQuizMessage::AnswerChanged(c) => self.answer = c.clone(),
            DictionaryQuizMessage::SubmitAnswer => self.submit(),
            DictionaryQuizMessage::Appeal => self.appeal_answer(),
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, DictionaryQuizMessage> {
        container(
            iced::widget::column![
                self.laps(),
                iced::widget::column![
                    text!("{}", self.view).size(54),
                    text!(
                        "{}",
                        if self.is_help {
                            self.correct.clone()
                        } else {
                            String::new()
                        }
                    ).size(20).align_y(alignment::Vertical::Center),
                ]
                .align_x(alignment::Horizontal::Center)
                .spacing(5),
                text_input("Перевод", &self.answer)
                    .size(28)
                    .width(250)
                    .on_input(DictionaryQuizMessage::AnswerChanged)
                    .on_submit(DictionaryQuizMessage::SubmitAnswer),
                row![
                    text!("{}", self.score.total.to_string()).size(25),
                    text!("{}", self.score.correct.to_string())
                        .size(25)
                        .color(iced::Color::from_rgb8(60, 170, 60)),
                    text!("{}", self.score.fail.to_string())
                        .color(iced::Color::from_rgb8(255, 79, 0))
                        .size(25),
                ]
                .spacing(DEFAULT_SPACING),
                row![
                    button("Закончить").on_press(DictionaryQuizMessage::Back),
                    self.appeal_button()
                ]
                .spacing(DEFAULT_SPACING),
            ]
            .spacing(DEFAULT_SPACING)
            .align_x(alignment::Horizontal::Center),
        )
        .center_y(Fill)
        .center_x(Fill)
        .into()
    }
    fn submit(&mut self) {
        if self.view == "---" {
            self.show_next();
            return;
        }

        if self.no_typing {
            self.no_type_submit();
        } else {
            self.default_submit();
        }
    }

    fn no_type_submit(&mut self) {
        if self.is_help {
            self.is_help = false;
            self.score.total += 1;
            self.show_next()
        }else {
            self.is_help = true;
        }
    }

    fn default_submit(&mut self) {
        if !self.is_help {
            self.score.total += 1;
        }
        if self.answer == self.correct
            || split_with_coma(self.correct.clone()).contains(&self.answer)
        {
            if self.is_help == false {
                self.score.correct += 1;
            }
            self.show_next()
        } else {
            self.score.fail += 1;
            self.is_help = true;
        }
    }

    fn update_set(&mut self) {
        self.current_set.append(&mut self.words.clone());
        self.current_set.shuffle(&mut rand::rng());
        if self.score.total != 0 {
            self.laps += 1;
        }
    }

    fn show_next(&mut self) {
        self.is_help = false;
        self.answer = String::new();

        if self.current_set.is_empty() {
            self.update_set();
        }

        let next = self.current_set.pop().unwrap();
        if self.reverse {
            self.view = next.value.clone();
            self.correct = next.key.clone();
        } else {
            self.view = next.key.clone();
            self.correct = next.value.clone();
        }
    }

    fn laps(&self) -> Element<'_, DictionaryQuizMessage> {
        let mut col = Row::new();
        for _ in 0..self.laps {
            col = col.push(iced::widget::container(space().height(15).width(15)).style(
                |x: &Theme| Style {
                    text_color: None,
                    background: Some(Color(x.palette().primary)),
                    border: Border {
                        color: Default::default(),
                        width: 0.0,
                        radius: Radius::new(5),
                    },
                    shadow: Default::default(),
                    snap: false,
                },
            ));
        }
        col.spacing(DEFAULT_SPACING).into()
    }

    fn appeal_button(&self) -> Element<'_, DictionaryQuizMessage> {
        if self.is_help && self.no_typing == false {
            return button("Апелляция")
                .on_press(DictionaryQuizMessage::Appeal)
                .into();
        }
        space().into()
    }

    fn appeal_answer(&mut self) {
        self.score.correct += 1;
        self.score.fail -= 1;
        self.answer = self.correct.clone();
        self.show_next()
    }
}
