use crate::data_provider::words::{delete_group, delete_word, update_group, update_word};
use crate::dictionary::DictionaryMessage::{
    ChangeDirection, DeleteGroup, EditGroup, SaveGroup, Test,
};
use crate::dictionary_test::DictionaryQuizState;
use crate::lang::{WordData, WordGroup};
use crate::word::WordState;
use crate::Page::Word;
use crate::{AppState, NavigatedPage, Page, RootMessage, DEFAULT_SPACING};
use chrono::{DateTime, TimeDelta, Utc};
use iced::alignment::Vertical::Center;
use iced::widget::button::Style;
use iced::widget::button::{danger, text};
use iced::widget::space::horizontal;
use iced::widget::*;
use iced::{Border, Color, Length, Shadow, Task};
use rand::random_range;
use rayon::iter::IndexedParallelIterator;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use DictionaryMessage::Back;

#[derive(Clone)]
pub struct DictionaryState {
    state: Arc<Mutex<AppState>>,
    include_map: Vec<bool>,
    tag_map: HashMap<String, bool>,
    reverse: bool,
    search: String,
    no_typing: bool,
    selected_group_index: usize,
    reverse_list: bool,
    auto_save_queue: HashMap<usize, DateTime<Utc>>,
    total_tags_list: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DictionaryMessage {
    Back,
    SetTags(usize, String),
    SetKey(usize, String),
    SetValue(usize, String),
    SubmitWord(usize),
    WordAction(usize),
    NewWord,
    Include(usize, bool),
    IncludeTag(String, bool),
    Test,
    ResetTags,
    SetReverse(bool),
    Search(String),
    SetTyping(bool),
    CreateGroup,
    EditGroup(String),
    SaveGroup,
    SelectGroup(usize),
    DeleteGroup,
    ChangeDirection,
    TrySave(usize),
}

impl NavigatedPage<DictionaryMessage> for DictionaryState {
    fn navigate(&self, message: &DictionaryMessage) -> Option<Page> {
        if let Back = message {
            return Some(Page::PreviousPage);
        }
        if let Test = message {
            if self.include_map.iter().any(|x| *x) {
                let mut words = vec![];
                let dict = &self.state.lock().unwrap().dictionary;

                words = self
                    .include_map
                    .iter()
                    .zip(0..self.include_map.len())
                    .filter(|(flag, _)| **flag)
                    .map(|(_, index)| dict[index].clone())
                    .collect();

                return Some(Page::DictionaryQuiz(DictionaryQuizState::new(
                    words,
                    self.reverse,
                    self.no_typing,
                )));
            }
        }
        if let DictionaryMessage::WordAction(index) = message {
            let word: WordData;
            {
                let state = self.state.lock().unwrap();
                let dict = &state.dictionary;
                word = dict[*index].clone();
            }
            if word.id != 0 {
                return Some(Word(WordState::new(word, *index, self.state.clone())));
            }
        }
        None
    }
}

impl DictionaryState {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        let len = state.lock().unwrap().dictionary.len();

        let mut result = DictionaryState {
            include_map: vec![false; len],
            selected_group_index: 0,
            state,
            tag_map: HashMap::new(),
            reverse: false,
            search: "".to_string(),
            no_typing: true,
            reverse_list: true,
            auto_save_queue: HashMap::new(),
            total_tags_list: vec![],
        };

        result.update_tags();

        result
    }

    pub fn update(&mut self, message: DictionaryMessage) -> Task<RootMessage> {
        match message {
            DictionaryMessage::NewWord => {
                let mut state = self.state.lock().unwrap();
                let mut word = WordData::new();
                word.group_id = state.word_groups[self.selected_group_index].id.clone();

                let dict = &mut state.dictionary;
                dict.push(word);
                self.include_map.push(false);
            }

            DictionaryMessage::SetKey(i, v) => {
                {
                    let dict = &mut self.state.lock().unwrap().dictionary;
                    dict[i].key = v;
                }
                return self.launch_auto_save_offset(i);
            }
            DictionaryMessage::SetValue(i, v) => {
                {
                    let dict = &mut self.state.lock().unwrap().dictionary;
                    dict[i].value = v
                }
                return self.launch_auto_save_offset(i);
            }
            DictionaryMessage::SetTags(i, mut v) => {
                {
                    let dict = &mut self.state.lock().unwrap().dictionary;

                    let current_tags_value = dict[i].tags.clone();

                    if v.ends_with(", ") && v.len() < current_tags_value.len() {
                        v = v[..v.len() - 2].to_string()
                    }


                    while v.contains(",,") {
                        let index = v.find(",,").unwrap();
                        v.remove(index);
                    }

                    dict.get_mut(i).unwrap().tags = v;
                }

                self.update_tags();
                return self.launch_auto_save_offset(i);
            }

            DictionaryMessage::WordAction(i) => {
                let state = &mut self.state.lock().unwrap();
                let dict = &mut state.dictionary;
                let word = dict.remove(i);
                self.include_map.remove(i);
                self.auto_save_queue.remove(&i);
                delete_word(&word, &state.connection)
            }
            DictionaryMessage::Include(i, b) => self.include_map[i] = b,
            DictionaryMessage::IncludeTag(t, v) => {
                let index: u32;
                {
                    let state = self.state.lock().unwrap();
                    index = state.word_groups[self.selected_group_index].id;
                }
                self.tag_map.insert(t, v);
                self.update_words_include(index)
            }

            DictionaryMessage::ResetTags => {
                self.tag_map.iter_mut().for_each(|(_, v)| *v = false);
                self.include_map.iter_mut().for_each(|x| *x = false)
            }
            DictionaryMessage::SetReverse(v) => self.reverse = v,
            DictionaryMessage::Search(s) => {
                self.search = s;
            }
            DictionaryMessage::SetTyping(b) => self.no_typing = b,
            DictionaryMessage::SubmitWord(i) => self.save_word(i),
            Back => {}
            Test => {}
            DictionaryMessage::CreateGroup => {
                let state = &mut self.state.lock().unwrap();

                state.word_groups.push(WordGroup {
                    id: 0,
                    name: format!("Группа слов {}", random_range(100..1000)),
                });
            }
            EditGroup(new) => {
                let state = &mut self.state.lock().unwrap();
                let group = state.word_groups.get_mut(self.selected_group_index);
                if let Some(group) = group {
                    group.name = new.clone();
                }
            }
            SaveGroup => {
                let state = &mut self.state.lock().unwrap();
                let connection = &state.connection;
                let group = &mut state
                    .word_groups
                    .get(self.selected_group_index)
                    .unwrap()
                    .clone();

                update_group(group, connection);
                state.word_groups[self.selected_group_index] = group.clone();
            }
            DictionaryMessage::SelectGroup(i) => {
                self.selected_group_index = i;
                let index: u32;
                {
                    let state = self.state.lock().unwrap();
                    index = state.word_groups[i].id;
                }
                self.update_words_include(index)
            }
            DeleteGroup => {
                if self.selected_group_index == 0 {
                    return Task::none();
                }
                let state = &mut self.state.lock().unwrap();

                if let Some(group) = state.word_groups.get(self.selected_group_index) {
                    let connection = &state.connection;

                    delete_group(group, connection);
                    state.word_groups.remove(self.selected_group_index);
                    self.selected_group_index = 0;
                }
            }
            ChangeDirection => {
                self.reverse_list = !self.reverse_list;
            }
            DictionaryMessage::TrySave(word_index) => {
                let now = Utc::now();
                if !self.auto_save_queue.contains_key(&word_index) {
                    return Task::none();
                }
                if self.auto_save_queue[&word_index] <= now {
                    self.auto_save_queue.remove(&word_index);
                    self.save_word(word_index);
                }
            }
        }

        Task::none()
    }

    fn save_word(&mut self, i: usize) {
        let state = &mut self.state.lock().unwrap();
        let connection = &state.connection;
        let word = &mut state.dictionary.get(i).unwrap().clone();

        update_word(word, &connection);
        state.dictionary[i] = word.clone();
    }

    fn launch_auto_save_offset(&mut self, index: usize) -> Task<RootMessage> {
        let save_time = Utc::now().add(TimeDelta::milliseconds(900));
        self.auto_save_queue.insert(index, save_time);
        let message = RootMessage::Dictionary(DictionaryMessage::TrySave(index));
        Task::perform(
            async { tokio::time::sleep(Duration::from_secs(1)).await },
            |_| message,
        )
    }

    pub fn view(&self) -> iced::Element<'_, DictionaryMessage> {
        container(
            row![
                iced::widget::column![
                    button("Назад").on_press(Back),
                    self.groups_panel(),
                    self.words_list(),
                    button("Добавить слово").on_press(DictionaryMessage::NewWord),
                ]
                .spacing(5),
                self.filters()
            ]
                .spacing(DEFAULT_SPACING),
        )
            .padding(10)
            .into()
    }

    fn words_list(&self) -> iced::Element<'_, DictionaryMessage> {
        let mut col = Column::new().width(Length::Fill);

        let mut range = (0..self.include_map.len()).collect::<Vec<_>>();
        let state = self.state.lock().unwrap();
        let group_id = state.word_groups[self.selected_group_index].id;
        let dict = &mut state.dictionary.clone();

        if self.reverse_list {
            dict.reverse()
        } else {
            range = range.iter().rev().map(|x| *x).collect::<Vec<usize>>();
        }

        for word in dict {
            let i = range.pop().unwrap();
            if !self.search.is_empty() {
                if word.key.contains(&self.search) == false
                    && word.value.contains(&self.search) == false
                {
                    continue;
                }
            }

            if word.group_id != group_id {
                continue;
            }

            let mut line = Row::new().width(Length::Fill).align_y(Center);
            line = line
                .push(
                    checkbox(self.include_map[i])
                        .on_toggle(move |b| DictionaryMessage::Include(i, b)),
                )
                .push(space().width(10));

            line = line.push(
                text_input("Слово", &word.key)
                    .size(20)
                    .width(Length::Fill)
                    .on_input(move |string| DictionaryMessage::SetKey(i, string))
                    .on_submit(DictionaryMessage::SubmitWord(i)),
            );
            line = line.push(
                text_input("Перевод", &word.value)
                    .size(20)
                    .width(Length::Fill)
                    .on_input(move |string| DictionaryMessage::SetValue(i, string))
                    .on_submit(DictionaryMessage::SubmitWord(i)),
            );
            line = line.push(
                text_input("Тэги", &word.tags)
                    .size(20)
                    .width(Length::Fill)
                    .on_input(move |string| DictionaryMessage::SetTags(i, string))
                    .on_submit(DictionaryMessage::SubmitWord(i)),
            );

            let line_button = || {
                let action = DictionaryMessage::WordAction(i);

                if word.id == 0 {
                    return button("-").on_press(action).style(|_x, _status| Style {
                        background: None,
                        text_color: Color::BLACK,
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: false,
                    });
                }

                button("")
                    .on_press(DictionaryMessage::WordAction(i))
                    .width(15)
            };

            line = line.push(line_button()).push(space().width(10));

            col = col.push(line);
        }

        scrollable(col).height(Length::Fill).into()
    }

    fn filters(&self) -> iced::Element<'_, DictionaryMessage> {
        let dict = &self.state.lock().unwrap().dictionary;

        iced::widget::column![
            text_input("Поиск", &self.search)
                .on_input(DictionaryMessage::Search)
                .width(Length::Fill),
            text!("Всего слов: {}", dict.len()),
            text!(
                "Выбрано слов: {}",
                self.include_map.iter().filter(|i| **i).count()
            ),
            self.tags_selector(),
            toggler(self.no_typing)
                .label("Без набора")
                .on_toggle(DictionaryMessage::SetTyping),
            toggler(self.reverse)
                .label("Обратный тест")
                .on_toggle(DictionaryMessage::SetReverse),
            button(text!("Тест").center().width(Length::Fill))
                .on_press(Test)
                .width(Length::Fill),
        ]
            .width(250)
            .spacing(DEFAULT_SPACING)
            .into()
    }

    fn tags_selector(&self) -> iced::Element<'_, DictionaryMessage> {
        let mut col = Column::new().width(Length::Fill);
        col = col.push(
            button("Сбросить")
                .on_press(DictionaryMessage::ResetTags)
                .style(|x: &Theme, _status| Style {
                    background: None,
                    text_color: x.palette().primary,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: false,
                }),
        );
        let mut sorted_tags = self.tag_map.iter().collect::<Vec<_>>();
        sorted_tags.sort();
        for tag in sorted_tags {
            col = col.push(
                checkbox(*tag.1)
                    .label(tag.0)
                    .on_toggle(|x1| DictionaryMessage::IncludeTag(tag.0.clone(), x1)),
            )
        }

        container(scrollable(col)).height(Length::Fill).into()
    }

    fn update_tags(&mut self) {
        let mut tags_list: HashSet<String> = HashSet::new();
        let dict = &self.state.lock().unwrap().dictionary;

        dict.iter().for_each(|element| {
            split_with_coma(element.tags.as_str())
                .iter()
                .for_each(|tag| {
                    tags_list.insert(tag.clone());
                });
        });


        let current_tags = self
            .tag_map
            .keys()
            .map(|k| k.clone().to_string())
            .collect::<Vec<String>>();

        current_tags
            .iter()
            .filter(|tag| !tags_list.contains(*tag))
            .for_each(|tag| {
                self.tag_map.remove(tag);
            });

        tags_list.iter().for_each(|tag| {
            self.tag_map.insert(tag.clone(), false);
        });
    }

    fn update_words_include(&mut self, group_id: u32) {
        let include_tags = self
            .tag_map
            .iter()
            .filter(|(_, value)| **value)
            .map(|(t, _)| t.clone())
            .collect::<Vec<String>>();

        if include_tags.is_empty() {
            self.include_map.iter_mut().for_each(|x| *x = false);
            return;
        }

        let dict = &self.state.lock().unwrap().dictionary;
        let time = Instant::now();

        self.include_map = dict
            .iter()
            .map(|word| (split_with_coma(word.tags.as_str()), word.group_id))
            .map(|(tags, word_group_id)| {
                tags.iter().all(|t| include_tags.contains(t)) && word_group_id == group_id
            })
            .collect();

        println!("Time {}", time.elapsed().as_micros());
    }

    fn groups_panel(&self) -> iced::Element<'_, DictionaryMessage> {
        let mut row = Row::new();
        row = row.push(
            button("+")
                .style(text)
                .on_press(DictionaryMessage::CreateGroup),
        );

        let state = &self.state.lock().unwrap();
        let groups = &state.word_groups;

        let mut index = 0;
        for group in groups {
            row = row.push(
                button(text!("{}", group.name.clone()))
                    .style(text)
                    .on_press(DictionaryMessage::SelectGroup(index)),
            );
            index = index + 1;
        }

        let group = state.word_groups[self.selected_group_index].clone();

        iced::widget::column![
            scrollable(row).width(Length::Fill).horizontal(),
            row![
                button("⇳").on_press(ChangeDirection),
                text_input("Название группы слов", &group.name)
                    .on_input(EditGroup)
                    .width(250)
                    .on_submit(SaveGroup),
                horizontal(),
                button("Удалить").style(danger).on_press(DeleteGroup),
            ]
            .spacing(DEFAULT_SPACING)
        ]
            .into()
    }
}

pub fn split_with_coma(ts: &str) -> Vec<String> {
    ts.split(',')
        .map(|ts| ts.to_lowercase().trim().to_string())
        .filter(|t| !t.is_empty())
        .collect::<Vec<String>>()
}

pub fn app_data_dir() -> PathBuf {
    let mut dir = dirs::data_dir().unwrap();
    dir.push("jap_learn");
    if !dir.exists() {
        fs::create_dir(dir.clone()).unwrap();
    }

    dir
}
