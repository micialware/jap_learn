use crate::data_provider::words::{delete_word, update_word};
use crate::lang::WordData;
use crate::Page::PreviousPage;
use crate::{AppState, NavigatedPage, Page, RootMessage, DEFAULT_SPACING};
use iced::widget::button::danger;
use iced::widget::{button, column, container, row, rule, scrollable, space, text, text_input};
use iced::{Element, Fill, Task};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WordState {
    state: Arc<Mutex<AppState>>,
    index: usize,
    word: WordData,
}

impl NavigatedPage<WordMessage> for WordState {
    fn navigate(&self, message: &WordMessage) -> Option<Page> {
        if let WordMessage::Back = message {
            Some(PreviousPage)
        } else {
            None
        }
    }
}

impl WordState {
    pub(crate) fn new(word: WordData, index: usize, state: Arc<Mutex<AppState>>) -> WordState {
        WordState { state, index, word }
    }
}

impl WordState {
    pub fn update(&mut self, message: WordMessage) -> Task<RootMessage> {
        match message {
            WordMessage::Back => {}
            WordMessage::Save => {
                let mut state = self.state.lock().unwrap();
                state.dictionary[self.index] = self.word.clone();
                update_word(&mut self.word, &state.connection);
                return Task::done(RootMessage::Word(WordMessage::Back));
            }
            WordMessage::Delete => {
                let mut state = self.state.lock().unwrap();
                state.dictionary.remove(self.index);
                delete_word(&self.word, &state.connection);
                return Task::done(RootMessage::Word(WordMessage::Back));
            }
            WordMessage::SetTags(n) => self.word.tags = n,
            WordMessage::SetKey(n) => {
                self.word.key = n;
            }
            WordMessage::SetValue(n) => {
                self.word.value = n;
            }
            WordMessage::SetAdditional(key, value) => match key.as_str() {
                _ => {
                    self.word.additional.insert(key, value.clone());
                }
            },
            WordMessage::AddAdditional(key) => {
                self.word.additional.insert(key, "".to_string());
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, WordMessage> {
        let mut fast_add = row![];
        if !self.word.additional.contains_key("reading") {
            fast_add = fast_add.push(
                button("Чтение")
                    .style(button::text)
                    .on_press(WordMessage::AddAdditional("reading".to_string())),
            );
        }
        if !self.word.additional.contains_key("description") {
            fast_add = fast_add.push(
                button("Описание")
                    .style(button::text)
                    .on_press(WordMessage::AddAdditional("description".to_string())),
            );
        }

        if !self.word.additional.contains_key("context") {
            fast_add = fast_add.push(
                button("В контексте")
                    .style(button::text)
                    .on_press(WordMessage::AddAdditional("context".to_string())),
            );
        }
        
        let mut col = iced::widget::column![
            button("Назад").on_press(WordMessage::Back),
            text!("Ключ"),
            text_input("key", &self.word.key).on_input(WordMessage::SetKey),
            text!("Значение"),
            text_input("value", &self.word.value).on_input(WordMessage::SetValue),
            text!("Теги"),
            text_input("tags", &self.word.tags).on_input(WordMessage::SetTags),
            rule::horizontal(2),
            scrollable(fast_add),
        ];

        for more in &self.word.additional {
            col = col.push(self.get_view_for_more(more));
        }
        container(
            column![
                col.spacing(DEFAULT_SPACING).width(Fill).height(Fill),
                row![
                    button("Сохранить").on_press(WordMessage::Save),
                    button("Удалить")
                        .style(danger)
                        .on_press(WordMessage::Delete),
                ]
                .spacing(DEFAULT_SPACING)
            ]
            .spacing(DEFAULT_SPACING),
        )
        .padding(DEFAULT_SPACING)
        .into()
    }

    fn get_view_for_more(&self, value: (&String, &String)) -> Element<'_, WordMessage> {
        match value.0.as_str() {
            "reading" => self.reading_field(value),
            "description" => self.description_field(value),
            "context" => self.context_field(value),
            _ => space().into(),
        }
    }

    fn reading_field(&self, value: (&String, &String)) -> Element<'_, WordMessage> {
        self.additional_field(value, "Чтение слова".to_string(), "reading".to_string())
    }

    fn description_field(&self, value: (&String, &String)) -> Element<'_, WordMessage> {
        self.additional_field(value, "Описание".to_string(), "description".to_string())
    }
    
    fn context_field(&self, value: (&String, &String)) -> Element<'_, WordMessage> {
        self.additional_field(value, "В контексте".to_string(), "context".to_string())
    }
    
    fn additional_field(&self, value: (&String, &String), name: String, id: String) -> Element<'_, WordMessage> {
        column![
            text!("{}", name),
            text_input(id.clone().as_str(), &value.1)
                .on_input(move |string| WordMessage::SetAdditional(id.clone(), string))
        ]
            .spacing(DEFAULT_SPACING)
            .into()
    }
}
#[derive(Debug, Clone)]
pub enum WordMessage {
    Save,
    Back,
    Delete,
    SetTags(String),
    SetKey(String),
    SetValue(String),
    AddAdditional(String),
    SetAdditional(String, String),
}
