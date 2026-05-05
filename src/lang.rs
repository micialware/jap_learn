use std::cmp::min;
use crate::data_provider::card_stats::{add_stat, delete_stat, load_stats_of_set, update_stat_score};
use crate::repetitions::CardSetSettings;
use crate::AppState;
use chrono::{DateTime, Utc};
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const MAX_HISTORY_LEN: usize = 20;
const MAX_HISTORY_LEN_PART: f32 = 0.33;
const MAX_SCORE: i32 = 25;
const FADE_PER_DAY: f32 = 0.95;
#[derive(Clone, Debug)]
pub struct KanaSet {
    name: String,
    pub(crate) chars_type: KanaType,
    pub(crate) dictionary: Vec<Vec<(String, String)>>,
    pub(crate) include_map: [bool; 10],
}

#[derive(Clone, Debug)]
pub enum KanaType {
    Hiragana,
    Katakana,
}

impl KanaSet {
    pub fn hiragana() -> Self {
        Self {
            name: "Хирагана".to_string(),
            chars_type: KanaType::Hiragana,
            dictionary: vec![
                vec![
                    (String::from("ぁ"), String::from("a")),
                    (String::from("ぃ"), String::from("i")),
                    (String::from("ぅ"), String::from("u")),
                    (String::from("ぇ"), String::from("e")),
                    (String::from("ぉ"), String::from("o")),
                ],
                vec![
                    (String::from("さ"), String::from("sa")),
                    (String::from("し"), String::from("shi")),
                    (String::from("す"), String::from("su")),
                    (String::from("せ"), String::from("se")),
                    (String::from("そ"), String::from("so")),
                ],
                vec![
                    (String::from("か"), String::from("ka")),
                    (String::from("き"), String::from("ki")),
                    (String::from("く"), String::from("ku")),
                    (String::from("け"), String::from("ke")),
                    (String::from("こ"), String::from("ko")),
                ],
                // Ряд «та» (たちつてと)
                vec![
                    (String::from("た"), String::from("ta")),
                    (String::from("ち"), String::from("chi")),
                    (String::from("つ"), String::from("tsu")),
                    (String::from("て"), String::from("te")),
                    (String::from("と"), String::from("to")),
                ],
                // Ряд «на» (なにぬねの)
                vec![
                    (String::from("な"), String::from("na")),
                    (String::from("に"), String::from("ni")),
                    (String::from("ぬ"), String::from("nu")),
                    (String::from("ね"), String::from("ne")),
                    (String::from("の"), String::from("no")),
                ],
                // Ряд «ха» (はひふへほ)
                vec![
                    (String::from("は"), String::from("ha")),
                    (String::from("ひ"), String::from("hi")),
                    (String::from("ふ"), String::from("fu")),
                    (String::from("へ"), String::from("he")),
                    (String::from("ほ"), String::from("ho")),
                ],
                // Ряд «ма» (まみむめも)
                vec![
                    (String::from("ま"), String::from("ma")),
                    (String::from("み"), String::from("mi")),
                    (String::from("む"), String::from("mu")),
                    (String::from("め"), String::from("me")),
                    (String::from("も"), String::from("mo")),
                ],
                // Ряд «я» (やゆよ) — только 3 символа
                vec![
                    (String::from("や"), String::from("ya")),
                    (String::from("ゆ"), String::from("yu")),
                    (String::from("よ"), String::from("yo")),
                ],
                // Ряд «ра» (らりるれろ)
                vec![
                    (String::from("ら"), String::from("ra")),
                    (String::from("り"), String::from("ri")),
                    (String::from("る"), String::from("ru")),
                    (String::from("れ"), String::from("re")),
                    (String::from("ろ"), String::from("ro")),
                ],
                vec![
                    (String::from("わ"), String::from("wa")),
                    (String::from("を"), String::from("wo")),
                    (String::from("ん"), String::from("n")),
                ],
            ],
            include_map: [true; 10],
        }
    }

    pub fn katakana() -> Self {
        Self {
            name: "Катакана".to_string(),
            chars_type: KanaType::Katakana,
            dictionary: vec![
                vec![
                    (String::from("ァ"), String::from("a")),
                    (String::from("ィ"), String::from("i")),
                    (String::from("ゥ"), String::from("u")),
                    (String::from("ェ"), String::from("e")),
                    (String::from("ォ"), String::from("o")),
                ],
                vec![
                    (String::from("サ"), String::from("sa")),
                    (String::from("シ"), String::from("shi")),
                    (String::from("ス"), String::from("su")),
                    (String::from("セ"), String::from("se")),
                    (String::from("ソ"), String::from("so")),
                ],
                vec![
                    (String::from("カ"), String::from("ka")),
                    (String::from("キ"), String::from("ki")),
                    (String::from("ク"), String::from("ku")),
                    (String::from("ケ"), String::from("ke")),
                    (String::from("コ"), String::from("ko")),
                ],
                // Ряд «та» (タチツテト)
                vec![
                    (String::from("タ"), String::from("ta")),
                    (String::from("チ"), String::from("chi")),
                    (String::from("ツ"), String::from("tsu")),
                    (String::from("テ"), String::from("te")),
                    (String::from("ト"), String::from("to")),
                ],
                // Ряд «на» (ナニヌネノ)
                vec![
                    (String::from("ナ"), String::from("na")),
                    (String::from("ニ"), String::from("ni")),
                    (String::from("ヌ"), String::from("nu")),
                    (String::from("ネ"), String::from("ne")),
                    (String::from("ノ"), String::from("no")),
                ],
                // Ряд «ха» (ハヒフヘホ)
                vec![
                    (String::from("ハ"), String::from("ha")),
                    (String::from("ヒ"), String::from("hi")),
                    (String::from("フ"), String::from("fu")),
                    (String::from("ヘ"), String::from("he")),
                    (String::from("ホ"), String::from("ho")),
                ],
                // Ряд «ма» (マミムメモ)
                vec![
                    (String::from("マ"), String::from("ma")),
                    (String::from("ミ"), String::from("mi")),
                    (String::from("ム"), String::from("mu")),
                    (String::from("メ"), String::from("me")),
                    (String::from("モ"), String::from("mo")),
                ],
                // Ряд «я» (ヤユヨ) — только 3 символа
                vec![
                    (String::from("ヤ"), String::from("ya")),
                    (String::from("ユ"), String::from("yu")),
                    (String::from("ヨ"), String::from("yo")),
                ],
                // Ряд «ра» (ラリルレロ)
                vec![
                    (String::from("ラ"), String::from("ra")),
                    (String::from("リ"), String::from("ri")),
                    (String::from("ル"), String::from("ru")),
                    (String::from("レ"), String::from("re")),
                    (String::from("ロ"), String::from("ro")),
                ],
                vec![
                    (String::from("ワ"), String::from("wa")),
                    (String::from("ヲ"), String::from("wo")),
                    (String::from("ン"), String::from("n")),
                ],
            ],
            include_map: [true; 10],
        }
    }

    /*    pub fn next(&mut self) -> (String, String) {
        let current_set = self.list();

        let mut rand = rand::rng();
        let index: u32 = rand.random();
        current_set[index as usize % current_set.len()].clone()
    }*/

    pub fn list(&self) -> Vec<(String, String)> {
        let mut current_set: Vec<(String, String)> = Vec::new();

        for i in 0..10 {
            if self.include_map[i] {
                self.dictionary[i].iter().for_each(|v| {
                    current_set.push(v.clone());
                });
            }
        }

        current_set
    }
}

impl Default for KanaSet {
    fn default() -> Self {
        KanaSet::hiragana()
    }
}
impl PartialEq<Self> for KanaSet {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordData {
    pub id: u32,
    pub key: String,
    pub value: String,
    pub tags: String,
    pub additional: HashMap<String, String>,
    pub group_id: u32
}

impl WordData {
    pub fn new() -> Self {
        Self {
            id: 0,
            key: String::new(),
            value: String::new(),
            tags: String::new(),
            additional: Default::default(),
            group_id: 1
        }
    }
}

pub struct WordGroup{
    pub id: u32,
    pub name: String,
}

#[derive(Clone, PartialEq)]
pub struct CardStatistics {
    pub id: u32,
    pub word_id: u32,
    pub set_id: u32,
    pub last_open: DateTime<Utc>,
    pub score: i32,
}

impl CardStatistics {
    pub fn update(&mut self, status: WordOpenMode) {
        match status {
            WordOpenMode::Easy => {
                self.score = (self.calculated_score() + 5.0).round() as i32;
            }
            WordOpenMode::Ok => {
                self.score = (self.calculated_score() + 2.0).round() as i32
            }
            WordOpenMode::Hard => {
                self.score = (self.calculated_score() - 1.0).round() as i32;
            }
            WordOpenMode::None => {
                self.score = (self.calculated_score() * 0.5) as i32;
            }
        }

        if self.score < 1 {
            self.score = 1
        } else if self.score > MAX_SCORE {
            self.score = MAX_SCORE
        }
        self.last_open = Utc::now();

    }

    pub fn calculated_score(&self) -> f32 {
        let time = Utc::now() - self.last_open;
        let days = time.num_days();
        let multiplier = FADE_PER_DAY.powi(days as i32);
         self.score as f32 * multiplier
    }
}

#[derive(Clone)]
pub enum WordOpenMode {
    Easy,
    Ok,
    Hard,
    None,
}

#[derive(Clone)]
pub struct CardSet {
    words: Vec<WordData>,
    set: Vec<CardStatistics>,
    last_weights: WeightedIndex<f32>,
    current_word_index: Option<usize>,
    generator: ThreadRng,
    state: Arc<Mutex<AppState>>,
    history: Vec<usize>
}

impl CardSet {
    pub fn new(settings: &CardSetSettings, state: Arc<Mutex<AppState>>) -> Self {
        let state_for = state.clone();
        let state_locked = state.lock().unwrap();

        let mut current_set = load_stats_of_set(&settings, &state_locked.connection);
        let last_list = settings.get_word_list(&state_locked);
        let saved_ids = current_set.iter().map(|l| l.word_id).collect::<Vec<u32>>();
        let word_ids = last_list.iter().map(|l| l.id).collect::<Vec<u32>>();
        for word in &last_list {
            if !saved_ids.contains(&word.id) {
                let mut new_statistic = CardStatistics {
                    id: 0,
                    word_id: word.id.clone(),
                    last_open: Utc::now(),
                    score: 1,
                    set_id: settings.id.clone(),
                };

                add_stat(&mut new_statistic, &state_locked.connection);

                current_set.push(new_statistic);
            }
        }

        let mut index = 0;
        for stat in current_set.clone() {
            if !word_ids.contains(&stat.word_id) {
                delete_stat(&stat, &state_locked.connection);
                current_set.remove(index);
            }
            index += 1;
        }

        let weights = current_set.iter().map(|s| (100.0 / s.calculated_score()).powf(2.0)).collect::<Vec<f32>>();
        let indexes = WeightedIndex::new(weights).unwrap();

        Self {
            set: current_set,
            words: last_list,
            last_weights: indexes,
            current_word_index: None,
            generator: rng(),
            state: state_for,
            history: vec![],
        }
    }


    pub fn next(&mut self) -> (WordData, CardStatistics) {
        let index = self.last_weights.sample(&mut self.generator);

        if self.history.contains(&index) {
            return self.next()
        }

        if self.history.len() == self.history_len() {
            self.history.remove(0);
        }
        self.history.push(index);
        self.current_word_index = Some(index);
        (self.words[index].clone(), self.set[index].clone())
    }

    pub fn open(&mut self, status: WordOpenMode) {
        if let None = self.current_word_index {
            return;
        }

        let word = &mut self.set[self.current_word_index.unwrap()];
        word.update(status);
        let new_weight = (100.0 / word.calculated_score()).powf(2.0);
        self.last_weights.update_weights(&[(self.current_word_index.unwrap(), &new_weight)]).unwrap();
        {
            update_stat_score(word, &self.state.lock().unwrap().connection)
        }
    }

    fn history_len(&self) -> usize {
        return min(MAX_HISTORY_LEN, (self.set.len() as f32 * MAX_HISTORY_LEN_PART) as usize);
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }
}
