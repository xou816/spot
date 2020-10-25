use super::models::{AlbumDescription};


#[derive(Clone, Debug)]
pub enum BrowserAction {
    SetContent(Vec<AlbumDescription>),
    AppendContent(Vec<AlbumDescription>)
}

#[derive(Clone, Debug)]
pub enum BrowserEvent {
    ContentSet,
    ContentAppended(usize)
}

pub struct BrowserState {
    pub page: u32,
    pub albums: Vec<AlbumDescription>
}

impl BrowserState {

    pub fn new() -> Self {
        Self {
            page: 1,
            albums: vec![]
        }
    }

    pub fn update_with(&mut self, action: BrowserAction) -> Option<BrowserEvent> {
        match action {
            BrowserAction::SetContent(content) if content != self.albums => {
                self.page = 1;
                self.albums = content;
                Some(BrowserEvent::ContentSet)
            },
            BrowserAction::AppendContent(mut content) => {
                self.page += 1;
                let append_index = self.albums.len();
                self.albums.append(content.as_mut());
                Some(BrowserEvent::ContentAppended(append_index))
            }
            _ => None
        }
    }
}
