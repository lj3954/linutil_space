use cosmic::{
    iced::Length,
    widget::{button, icon::Named, nav_bar, ListColumn},
    Element,
};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::app::Message;

const COMMAND_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/linutil/src/commands");

pub struct Tabs {
    tabs: Vec<Tab>,
    directory_structure: Vec<usize>,
}

impl Tabs {
    pub fn new() -> Self {
        let temp_dir = create_temp_dir();
        COMMAND_DIR
            .extract(&temp_dir)
            .expect("Failed to extract scripts into temp dir");

        let tab_files = TabList::get_tabs(&temp_dir);
        let tabs = tab_files
            .into_iter()
            .map(|path| {
                let directory = path.parent().unwrap().to_owned();
                let data = std::fs::read_to_string(path).expect("Failed to read tab data");
                let mut tab_data: Tab = toml::from_str(&data).expect("Failed to parse tab data");

                filter_entries(&mut tab_data.data, &directory);
                tab_data
            })
            .collect();

        Self {
            tabs,
            directory_structure: vec![],
        }
    }
    pub fn add_list(&mut self, nav: &mut nav_bar::Model) {
        self.tabs.iter_mut().enumerate().for_each(|(index, tab)| {
            nav.insert()
                .text(std::mem::take(&mut tab.name))
                .data::<usize>(index);
        });
    }
    pub fn reset_structure(&mut self) {
        self.directory_structure.clear();
    }
    pub fn view(&self, tab: usize) -> Element<Message> {
        let mut visible_items = self.tabs[tab].data.as_slice();
        for index in &self.directory_structure {
            visible_items = visible_items[*index].entries.as_ref().unwrap();
        }

        visible_items
            .iter()
            .enumerate()
            .fold(ListColumn::new(), |list, (index, entry)| {
                let mut button = button::text(entry.name.clone()).width(Length::Fill);
                if !entry.description.is_empty() {
                    button = button.tooltip(entry.description.clone());
                }

                let message = if entry.entries.is_some() {
                    Message::EnterDirectory(index)
                } else if let Some(command) = &entry.command {
                    Message::None
                } else if let Some(script) = &entry.script {
                    Message::None
                } else {
                    panic!("Entry {:?} {index} has no data", self.directory_structure);
                };

                let icon = if entry.entries.is_some() {
                    "folder"
                } else {
                    "utilities-terminal"
                };

                button = button.on_press(message).leading_icon(Named::new(icon));
                if entry.entries.is_some() {
                    button = button.on_press(Message::EnterDirectory(index));
                }
                list.add(button)
            })
            .into()
    }
    pub fn enter_directory(&mut self, index: usize) {
        self.directory_structure.push(index);
    }
}

fn create_temp_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir().join("linutil_scripts");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).expect("Failed to remove temp directory");
    }
    std::fs::create_dir(&temp_dir).expect("Failed to create temp directory");
    temp_dir
}

fn filter_entries(entries: &mut Vec<Entry>, command_dir: &Path) {
    entries.retain_mut(|entry| {
        if !entry.is_supported() {
            return false;
        }
        if let Some(script_path) = &mut entry.script {
            *script_path = command_dir.join(script_path.as_path());
        }
        if let Some(entries) = &mut entry.entries {
            filter_entries(entries, command_dir);
            !entries.is_empty()
        } else {
            true
        }
    });
}

#[derive(Deserialize)]
struct Tab {
    name: String,
    data: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    description: String,
    #[serde(default)]
    preconditions: Option<Vec<Precondition>>,
    #[serde(default)]
    entries: Option<Vec<Entry>>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    script: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Precondition {
    // If true, the data must be contained within the list of values.
    // Otherwise, the data must not be contained within the list of values
    matches: bool,
    data: SystemDataType,
    values: Vec<String>,
}

#[derive(Deserialize)]
enum SystemDataType {
    #[serde(rename = "environment")]
    Environment(String),
    #[serde(rename = "file")]
    File(PathBuf),
    #[serde(rename = "command_exists")]
    CommandExists,
}

impl Entry {
    fn is_supported(&self) -> bool {
        self.preconditions.as_deref().map_or(true, |preconditions| {
            preconditions.iter().all(
                |Precondition {
                     matches,
                     data,
                     values,
                 }| {
                    match data {
                        SystemDataType::Environment(var_name) => std::env::var(var_name)
                            .map_or(false, |var| values.contains(&var) == *matches),
                        SystemDataType::File(path) => {
                            std::fs::read_to_string(path).map_or(false, |data| {
                                values
                                    .iter()
                                    .any(|matching_value| data.contains(matching_value))
                                    == *matches
                            })
                        }
                        SystemDataType::CommandExists => values
                            .iter()
                            .all(|command| which::which(command).is_ok() == *matches),
                    }
                },
            )
        })
    }
}

#[derive(Deserialize)]
struct TabList {
    directories: Vec<PathBuf>,
}
impl TabList {
    fn get_tabs(command_dir: &Path) -> Vec<PathBuf> {
        let tab_files = std::fs::read_to_string(command_dir.join("tabs.toml"))
            .expect("Failed to read tabs.toml");
        let data: Self = toml::from_str(&tab_files).expect("Failed to parse tabs.toml");

        data.directories
            .into_iter()
            .map(|path| command_dir.join(path).join("tab_data.toml"))
            .collect()
    }
}
