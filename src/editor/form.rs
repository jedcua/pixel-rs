//! The forms of the palette dialogs.

use pxl::{Document, PaletteEntry, Rgba};

/// Keys that the add form proposes, in order. The list has no keys that look
/// alike, such as `0` and `O`, or `1`, `l`, and `I`.
const KEY_PROPOSALS: &str = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";

/// The maximum length of the color text: `#` and 8 hex digits.
const COLOR_TEXT_MAX: usize = 9;

/// The maximum length of the label text, in chars.
const LABEL_TEXT_MAX: usize = 32;

/// A text field of the entry form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Key,
    Color,
    Label,
}

/// The form to add a palette entry or to change one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryForm {
    /// The entry to change, or `None` to add an entry.
    pub index: Option<usize>,
    pub key: Option<char>,
    pub color: String,
    pub label: String,
    pub field: Field,
    pub error: Option<String>,
}

impl EntryForm {
    /// Makes a form to add an entry. It proposes a free key and the color of
    /// the entry `selected`.
    pub fn add(document: &Document, selected: usize) -> Self {
        let key = KEY_PROPOSALS
            .chars()
            .find(|&key| document.palette().iter().all(|entry| entry.key != key));
        EntryForm {
            index: None,
            key,
            color: document.palette()[selected].color.to_string(),
            label: String::new(),
            field: Field::Color,
            error: None,
        }
    }

    /// Makes a form to change the entry `index`.
    pub fn edit(document: &Document, index: usize) -> Self {
        let entry = &document.palette()[index];
        EntryForm {
            index: Some(index),
            key: Some(entry.key),
            color: entry.color.to_string(),
            label: entry.label.clone().unwrap_or_default(),
            field: Field::Color,
            error: None,
        }
    }

    /// Types `typed` into the active field. The key field holds one char, so
    /// a new char replaces the old one. The color and label fields ignore a
    /// char after their maximum length.
    pub fn type_char(&mut self, typed: char) {
        match self.field {
            Field::Key => self.key = Some(typed),
            Field::Color if self.color.len() < COLOR_TEXT_MAX => self.color.push(typed),
            Field::Label if self.label.chars().count() < LABEL_TEXT_MAX => self.label.push(typed),
            Field::Color | Field::Label => {}
        }
    }

    /// Deletes the last char of the active field.
    pub fn backspace(&mut self) {
        match self.field {
            Field::Key => self.key = None,
            Field::Color => {
                self.color.pop();
            }
            Field::Label => {
                self.label.pop();
            }
        }
    }

    /// Moves the focus to the next field. The last field goes back to the
    /// first.
    pub fn next_field(&mut self) {
        self.field = match self.field {
            Field::Key => Field::Color,
            Field::Color => Field::Label,
            Field::Label => Field::Key,
        };
    }

    /// Returns the entry that the form describes. A blank label gives no
    /// label.
    ///
    /// # Errors
    ///
    /// Returns a message for the user if a field is empty or not valid.
    pub fn entry(&self) -> Result<PaletteEntry, String> {
        let key = self.key.ok_or("Key is empty. Type one char.")?;
        let color = Rgba::from_lenient_hex(&self.color)
            .ok_or("Color is not valid. Type #RRGGBB or #RRGGBBAA.")?;
        let label = self.label.trim();
        Ok(PaletteEntry {
            key,
            color,
            label: (!label.is_empty()).then(|| label.to_owned()),
        })
    }
}

/// The form to remove a palette entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveForm {
    pub index: usize,
    /// The number of pixels that use the entry.
    pub usage: usize,
    /// The key of the entry that gets the pixels of the removed entry.
    pub replacement: Option<char>,
    pub error: Option<String>,
}

impl RemoveForm {
    /// Makes a form to remove the entry `index`. It proposes the first fully
    /// transparent entry as the replacement, or else the first other entry.
    pub fn new(document: &Document, index: usize) -> Self {
        let others = || {
            document
                .palette()
                .iter()
                .enumerate()
                .filter(move |&(other, _)| other != index)
        };
        let replacement = others()
            .find(|(_, entry)| entry.color.0[3] == 0)
            .or_else(|| others().next())
            .map(|(_, entry)| entry.key);
        RemoveForm {
            index,
            usage: document.usage(index),
            replacement,
            error: None,
        }
    }

    /// Returns the palette index of the replacement.
    ///
    /// # Errors
    ///
    /// Returns a message for the user if the key is empty or not in the
    /// palette.
    pub fn replacement_index(&self, document: &Document) -> Result<usize, String> {
        let key = self
            .replacement
            .ok_or("Replacement key is empty. Type a palette key.")?;
        document
            .palette()
            .iter()
            .position(|entry| entry.key == key)
            .ok_or_else(|| format!("Key `{key}` is not in the palette. Type a palette key."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Document {
        Document::parse("A - #11223344\n. - #00000000\nC - #FFFFFFFF\n\nA.C\n").unwrap()
    }

    #[test]
    fn add_form_proposes_a_free_key_and_the_selected_color() {
        let form = EntryForm::add(&document(), 2);
        assert_eq!(form.key, Some('B'));
        assert_eq!(form.color, "#FFFFFFFF");
    }

    #[test]
    fn entry_form_reports_empty_and_bad_fields() {
        let mut form = EntryForm::edit(&document(), 0);
        form.field = Field::Key;
        form.backspace();
        assert!(form.entry().unwrap_err().starts_with("Key is empty"));
        form.type_char('Z');
        form.next_field();
        form.backspace();
        assert!(form.entry().unwrap_err().starts_with("Color is not valid"));
    }

    #[test]
    fn entry_form_makes_a_label_only_if_the_text_is_not_blank() {
        let mut form = EntryForm::edit(&document(), 0);
        assert_eq!(form.entry().unwrap().label, None);
        form.field = Field::Label;
        for typed in "  sky blue ".chars() {
            form.type_char(typed);
        }
        assert_eq!(form.entry().unwrap().label.as_deref(), Some("sky blue"));
        form.label = "   ".into();
        assert_eq!(form.entry().unwrap().label, None);
    }

    #[test]
    fn label_field_stops_at_the_maximum_length() {
        let mut form = EntryForm::add(&document(), 0);
        form.field = Field::Label;
        for _ in 0..LABEL_TEXT_MAX + 5 {
            form.type_char('x');
        }
        assert_eq!(form.label.chars().count(), LABEL_TEXT_MAX);
    }

    #[test]
    fn remove_form_proposes_a_transparent_replacement() {
        let doc = document();
        let form = RemoveForm::new(&doc, 0);
        assert_eq!((form.usage, form.replacement), (1, Some('.')));
        assert_eq!(form.replacement_index(&doc), Ok(1));
        let form = RemoveForm::new(&doc, 1);
        assert_eq!(form.replacement, Some('A'));
    }
}
