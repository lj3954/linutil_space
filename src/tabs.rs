use cosmic::{
    iced::Length,
    widget::{self, button, icon::Named, nav_bar, ListColumn},
    Element,
};
use ego_tree::NodeId;
use linutil_core::Tab;

use crate::app::Message;

pub struct Tabs {
    tabs: Vec<Tab>,
    directory_structure: Vec<NodeId>,
}

impl Tabs {
    pub fn new() -> Self {
        let tabs = linutil_core::get_tabs(true);
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
        let tree = &self.tabs[tab].tree;
        let visible_items = if let Some(last) = self.directory_structure.last() {
            tree.get(*last).unwrap()
        } else {
            tree.root()
        };

        let list = visible_items
            .children()
            .fold(ListColumn::new(), |list, entry| {
                let node = entry.value().clone();
                let id = entry.id();
                let is_directory = entry.has_children();

                let mut button = button::text(node.name).width(Length::Fill);
                if !node.description.is_empty() {
                    button = button.tooltip(node.description);
                }

                let message = if is_directory {
                    Message::EnterDirectory(id)
                } else {
                    Message::None
                };

                let icon = if is_directory {
                    "folder"
                } else {
                    "utilities-terminal"
                };

                button = button.on_press(message).leading_icon(Named::new(icon));
                list.add(button)
            });
        widget::scrollable(list).into()
    }
    pub fn enter_directory(&mut self, node: NodeId) {
        self.directory_structure.push(node);
    }
}
