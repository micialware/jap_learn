use std::io;
use crate::data_provider::settings::set_setting;
use crate::sync::SyncMessage::NextAnimation;
use crate::Page::PreviousPage;
use crate::{AppState, NavigatedPage, Page, RootMessage, DEFAULT_SPACING};
use iced::widget::container::rounded_box;
use iced::widget::{button, column, container, progress_bar, row, space, text};
use iced::{Element, Fill, Font, Left, Task};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zstd::{Encoder, DEFAULT_COMPRESSION_LEVEL};
use crate::dictionary::app_data_dir;

const API_URL: &str = "https://learning.micialware.ru/";
/*const API_URL: &str = "http://localhost:8089/";*/

#[derive(Clone)]
pub enum SyncMessage {
    Back,
    CopyKey,
    KeyCopied,
    IdReceived(String),
    InitSync,
    GetKey,
    Send,
    GetLast,
    NextAnimation,
    NetworkFinished,
}
#[derive(Clone)]
pub struct SyncState {
    state: Arc<Mutex<AppState>>,
    frozen: bool,
    progress: f32,
}

impl NavigatedPage<SyncMessage> for SyncState {
    fn navigate(&self, message: &SyncMessage) -> Option<Page> {
        if let SyncMessage::Back = message
            && !self.frozen
        {
            Some(PreviousPage)
        } else {
            None
        }
    }
}

impl SyncState {
    pub fn new(state: Arc<Mutex<AppState>>) -> SyncState {
        Self {
            state,
            frozen: false,
            progress: 0.0,
        }
    }

    pub fn update(&mut self, message: SyncMessage) -> Task<RootMessage> {
        match message {
            SyncMessage::Back => {}
            SyncMessage::CopyKey => {
                let state = self.state.lock().unwrap();
                let key = state.sync_data.key.clone().unwrap();
                return iced::clipboard::write(key)
                    .map(|_val: String| RootMessage::Sync(SyncMessage::KeyCopied));
            }
            SyncMessage::InitSync => {
                return Task::perform(first_sync(), |id| {
                    RootMessage::Sync(SyncMessage::IdReceived(id))
                });
            }
            SyncMessage::GetKey => {
                return iced::clipboard::read().map(|key| {
                    RootMessage::Sync(SyncMessage::IdReceived(key.unwrap_or_else(String::new)))
                });
            }
            SyncMessage::KeyCopied => {}
            SyncMessage::IdReceived(new_id) => {
                if validate_id(&new_id) == false {
                    return Task::none();
                }
                let mut state = self.state.lock().unwrap();
                state.sync_data.key = Some(new_id.clone());
                set_setting("SYNC_KEY".to_string(), new_id, &state.connection);
            }
            SyncMessage::Send => {
                self.prepare_to_db_interaction();

                let id = self.state.lock().unwrap().sync_data.key.clone().unwrap();
                let tasks = Task::batch([
                    Task::perform(
                        async { tokio::time::sleep(Duration::from_millis(200)).await },
                        |_| RootMessage::Sync(NextAnimation),
                    ),
                    Task::perform(send_data(id), |_| {
                        RootMessage::Sync(SyncMessage::NetworkFinished)
                    }),
                ]);

                return tasks;
            }
            SyncMessage::GetLast => self.prepare_to_db_interaction(),
            NextAnimation => {
                self.progress += 1.0;
                if self.progress > 5.0 {
                    self.progress = 0.0;
                }
                if self.frozen {
                    return Task::perform(
                        async { tokio::time::sleep(Duration::from_millis(200)).await },
                        |_| RootMessage::Sync(NextAnimation),
                    );
                }
            }
            SyncMessage::NetworkFinished => {
                self.frozen = false;
            }
        }
        Task::none()
    }
    pub fn view(&self) -> Element<'_, SyncMessage> {
        container(
            column![
                button("Назад").on_press(SyncMessage::Back),
                row![self.sync_column(), self.app_updater()].spacing(DEFAULT_SPACING)
            ]
            .align_x(Left)
            .width(Fill)
            .height(Fill)
            .spacing(DEFAULT_SPACING),
        )
        .center_x(Fill)
        .padding(10)
        .into()
    }

    fn sync_column(&self) -> Element<'_, SyncMessage> {
        let network_view: Element<'_, SyncMessage> = if self.frozen {
            progress_bar(0.0..=5.0, self.progress).into()
        } else {
            space().into()
        };
        if let Some(key) = self.state.lock().unwrap().sync_data.key.clone() {
            column![
                text!("Ваш ключ синхронизации"),
                container(container(text!("{}", key).size(20).font(Font::MONOSPACE)).padding(3))
                    .style(rounded_box),
                button("Скопировать в буффер обмена").on_press(SyncMessage::CopyKey),
                row![
                    button("↑ Отправить").on_press(SyncMessage::Send),
                    button("↓ Скачать").on_press(SyncMessage::GetLast)
                ]
                .spacing(DEFAULT_SPACING),
                container(network_view).width(Fill),
            ]
            .spacing(DEFAULT_SPACING)
            .width(Fill)
            .into()
        } else {
            column![
                button("Создать сохранение").on_press(SyncMessage::InitSync),
                button("Вставить ключ из буффера").on_press(SyncMessage::GetKey),
            ]
            .spacing(DEFAULT_SPACING)
            .width(Fill)
            .into()
        }
    }

    fn app_updater(&self) -> Element<'_, SyncMessage> {
        column![].width(Fill).into()
    }

    fn prepare_to_db_interaction(&mut self) {
        if self.frozen {
            return;
        }
        self.frozen = true;
        self.progress = 0.0;
        let state = self.state.lock().unwrap();
        state
            .connection
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            .unwrap_or_else(|e| {
                eprintln!("Ошибка при checkpoint WAL: {e}");
            });
    }
}

async fn send_data(id: String) {
    let path = app_data_dir();
    let db_file = path.join("data.db");
    let data = tokio::fs::read(db_file).await.unwrap();
    let data = compress(data);

    let api_url = API_URL.to_string();
    let id_url = format!("{api_url}upload/{id}");

    let form = reqwest::multipart::Form::new().part("db", reqwest::multipart::Part::bytes(data));
    let client = reqwest::Client::new();
    client.post(&id_url).multipart(form).send().await.unwrap();
}

fn compress(data: Vec<u8>) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new(), DEFAULT_COMPRESSION_LEVEL).unwrap();
    io::copy(&mut &data[..], &mut encoder).unwrap();
    let compressed = encoder.finish().unwrap();
    compressed
}

async fn first_sync() -> String {
    let id_url = API_URL.to_string() + "generate";

    let client = reqwest::Client::new();
    let id = client
        .get(&id_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("id: {}", id);
    id
}

#[inline]
fn validate_id(id: &str) -> bool {
    id.len() == 24 && id.chars().all(|c| c.is_ascii_alphanumeric())
}
