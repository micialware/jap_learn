use crate::data_provider::card_sets::{delete_set, update_card_set};
use crate::lang::WordData;
use crate::repetition::RepetitionState;
use crate::Page::{PreviousPage, Repetition};
use crate::{AppState, NavigatedPage, Page, RootMessage, DEFAULT_SPACING};
use iced::widget::button::danger;
pub use iced::widget::button::{Catalog, Style};
use iced::widget::{button, column, container, row, scrollable, space, text, text_input, Column};
use iced::{Border, Center, Element, Fill, Left, Length, Shadow, Task, Theme};
use rhai::{Engine, Scope};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RepetitionsState {
    selected_set: Option<usize>,
    correct_filters: Vec<bool>,
    pub state: Arc<Mutex<AppState>>,
}

impl NavigatedPage<RepetitionsMessage> for RepetitionsState {
    fn navigate(&self, message: &RepetitionsMessage) -> Option<Page> {
        if let RepetitionsMessage::Back = message {
            Some(PreviousPage)
        }
        else if let RepetitionsMessage::GoToRepetition = message {
            let clone = self.state.clone();
            let card_set;
            {
                card_set = self.state.lock().unwrap().card_sets[self.selected_set.unwrap()].clone();
            }
            Some(Repetition(RepetitionState::new(card_set, clone) ))
        } else {
            None
        }
    }
}

impl RepetitionsState {
    pub(crate) fn new(state: Arc<Mutex<AppState>>) -> RepetitionsState {
        let count = state.lock().unwrap().card_sets.len();
        RepetitionsState {
            selected_set: None,
            correct_filters: vec![true; count],
            state,
        }
    }
}

impl RepetitionsState {
    pub fn update(&mut self, message: RepetitionsMessage) -> Task<RootMessage> {
        let mut state = self.state.lock().unwrap();
        match message {
            RepetitionsMessage::Next => {}
            RepetitionsMessage::Back => {}
            RepetitionsMessage::GoToRepetition => {}
            RepetitionsMessage::CreateSet => {
                let index = state.card_sets.len() + 1;
                state
                    .card_sets
                    .push(CardSetSettings::with_name(format!("Card set #{}", index)));
                self.correct_filters.push(true);
            }
            RepetitionsMessage::DeleteSet => {
                let set = state.card_sets.remove(self.selected_set.unwrap());
                self.correct_filters.remove(self.selected_set.unwrap());
                self.selected_set = None;
                delete_set(&set, &state.connection);
            }
            RepetitionsMessage::SelectSet(index) => {
                self.selected_set = Some(index);
            }
            RepetitionsMessage::SetName(new) => {
                state.card_sets[self.selected_set.unwrap()].name = new;
            }
            RepetitionsMessage::Save => {
                for i in 0..state.card_sets.len() {
                    self.correct_filters[i] = state.card_sets[i].check_filter();
                }

                let word = &mut state.card_sets[self.selected_set.unwrap()].clone();

                update_card_set(word, &state.connection);

                state.card_sets[self.selected_set.unwrap()] = word.clone();
            }
            RepetitionsMessage::SetForward(new) => {
                state.card_sets[self.selected_set.unwrap()].forward = new;
            }
            RepetitionsMessage::SetBackward(new) => {
                state.card_sets[self.selected_set.unwrap()].backward = new;
            }
            RepetitionsMessage::SetFilter(new) => {
                state.card_sets[self.selected_set.unwrap()].filter = new;
            }
            RepetitionsMessage::TryFilter => {
                let set = state.card_sets.get(self.selected_set.unwrap()).unwrap();
                let count = set.get_word_list(&state).len();
                state.card_sets[self.selected_set.unwrap()].count = Some(count);
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, RepetitionsMessage> {
        container(
            iced::widget::column![
                button("Назад").on_press(RepetitionsMessage::Back),
                row![
                    column![
                        scrollable(self.sets_list()).height(Fill),
                        button("Добавить")
                            .width(Fill)
                            .on_press(RepetitionsMessage::CreateSet),
                    ]
                    .spacing(DEFAULT_SPACING)
                    .width(Length::FillPortion(1)),
                    self.selected_set_view(),
                    self.launch_button()
                ]
                .align_y(Center)
                .padding(10)
                .spacing(DEFAULT_SPACING)
                .width(Fill)
                .height(Fill)
            ]
            .align_x(Left)
            .width(Fill),
        )
        .center_x(Fill)
        .padding(10)
        .into()
    }

    fn launch_button(&self) -> Element<'_, RepetitionsMessage> {
        if let Some(_) = self.selected_set {
            return button(text!("▷").height(Fill).center())
                .height(200)
                .on_press(RepetitionsMessage::GoToRepetition)
                .into();
        }
        space().into()
    }

    fn selected_set_view(&self) -> Element<'_, RepetitionsMessage> {
        if let Some(index) = self.selected_set {
            let sets = &self.state.lock().unwrap().card_sets;

            return column![
                scrollable(
                    column![
                        text_input("Название набора", &sets[index].name)
                            .on_input(RepetitionsMessage::SetName),
                        text_input("Передняя сторона", &sets[index].forward)
                            .on_input(RepetitionsMessage::SetForward),
                        text_input("Задняя сторона", &sets[index].backward)
                            .on_input(RepetitionsMessage::SetBackward),
                        text!("Фильтр"),
                        text_input("", &sets[index].filter).on_input(RepetitionsMessage::SetFilter),
                        button("Проверить фильтр").on_press(RepetitionsMessage::TryFilter),
                        self.count_view(&sets[index])
                    ]
                    .spacing(DEFAULT_SPACING)
                )
                .height(Fill),
                row![
                    button("Сохранить").on_press(RepetitionsMessage::Save),
                    button("Удалить")
                        .style(danger)
                        .on_press(RepetitionsMessage::DeleteSet)
                ]
                .spacing(DEFAULT_SPACING),
            ]
            .spacing(DEFAULT_SPACING)
            .width(Length::FillPortion(2))
            .into();
        }
        space().width(Length::FillPortion(2)).into()
    }

    fn count_view(&self, set: &CardSetSettings) -> Element<'_, RepetitionsMessage> {
        if let Some(count) = set.count {
            return text!("Колличество слов: {}", count).into();
        }
        space().into()
    }

    fn sets_list(&self) -> Column<'_, RepetitionsMessage> {
        let mut column = Column::new();
        let mut i = 0;
        let sets = &self.state.lock().unwrap().card_sets;
        for set in sets {
            column = column.push(
                button(text!("{}", set.name.clone()))
                    .on_press_with(move || RepetitionsMessage::SelectSet(i.clone()))
                    .style(move |_x: &Theme, _status| Style {
                        background: None,
                        text_color: if self.correct_filters[i.clone()] {
                            _x.palette().text
                        } else {
                            _x.palette().warning
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: false,
                    }),
            );
            i += 1;
        }

        column
    }
}

#[derive(Debug, Clone)]
pub enum RepetitionsMessage {
    Next,
    Back,
    GoToRepetition,
    CreateSet,
    DeleteSet,
    SetName(String),
    SelectSet(usize),
    Save,
    SetForward(String),
    SetBackward(String),
    SetFilter(String),
    TryFilter,
}

#[derive(Debug, Clone)]
pub struct CardSetSettings {
    pub id: u32,
    pub name: String,
    pub forward: String,
    pub backward: String,
    pub filter: String,
    pub count: Option<usize>,
}

impl CardSetSettings {
    fn with_name(name: String) -> CardSetSettings {
        CardSetSettings {
            id: 0,
            name,
            forward: "".to_string(),
            backward: "".to_string(),
            filter: "true".to_string(),
            count: None,
        }
    }

    fn check_filter(&self) -> bool {
        let engine = Engine::new();
        let ast = engine.compile(&self.filter);
        ast.is_ok()
    }

    pub fn get_word_list(&self, state: &AppState) -> Vec<WordData> {
        let mut list = vec![];
        let engine = Engine::new();
        let ast = engine.compile(&self.filter);
        if ast.is_err() {
            return list;
        }

        let ast = ast.unwrap();

        for word in &state.dictionary {
            let mut more = rhai::Map::new();
            for iced in &word.additional {
                more.insert(iced.0.clone().into(), iced.1.clone().into());
            }
            let mut scope = Scope::new();
            scope
                .push_constant("key", word.key.clone())
                .push_constant("value", word.value.clone())
                .push_constant("tags", word.tags.clone())
                .push_constant("more", more);

            let result = engine.eval_ast_with_scope::<bool>(&mut scope, &ast);
            if result.is_ok() && result.unwrap() {
                list.push(word.clone());
            }
        }

        list
    }

    pub fn require_speech(&self) -> bool {
        return self.forward == "speech" || self.backward == "speech" 
    }
}
