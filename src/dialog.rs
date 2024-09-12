// The role for a turn
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Role {
    System,
    User,
    Assistant,
    #[allow(dead_code)]
    IPython,
}

// A turn is one interaction of a role's conversation content
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Turn {
    pub role: Role,
    pub content: String,
}

// A dialog is a collection of turns uniquely identified by a thread
#[derive(Debug, PartialEq, Eq)]
pub struct Dialog {
    thread: String,
    turns: Vec<Turn>,
}

// A Dialog is a collection of communication turns.
impl Dialog {
    // Allocate a new Dialog with the given thread
    pub fn new(thread: String) -> Dialog {
        Dialog { thread, turns: Vec::new() }
    }

    // Add a conversation turn
    pub fn add(&mut self, role: Role, content: String) {
        self.turns.push(Turn { role, content });
    }

    // Method to format the dialog as a string
    pub fn format(&self) -> String {
        let mut formatted_dialog = String::new();

        // Add the metadata header
        formatted_dialog.push_str("<|begin_of_text|>\n");

        // Add each turn in the dialog
        for turn in self.turns.iter() {
            formatted_dialog.push_str(&format!(
                "<|start_header_id|>{:?}<|end_header_id|>\n\n{}<|eot_id|>\n",
                turn.role,
                turn.content
            ));
        }

	// Llama 3.1 docs state that a new header of type Assistant be added to the end
        // of the formatted text. This is not stored in the Dialog. Instead, when the API
        // returns an asistant message, it is added via Dialog.add(). However, to prompt
        // the assistant, we add a header id with the Assistant message.
        formatted_dialog.push_str("<|start_header_id|>Assistant<|end_header_id|>\n\n");

        formatted_dialog
    }
}
