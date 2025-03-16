// The role for a turn
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Role {
    model,
    user,
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

// `gemma-3`:
// <start_of_turn>user
// What is Cramer's Rule?<end_of_turn>
// <start_of_turn>model
// Cramer's Rule is ...<end_of_turn>

        // Add each turn in the dialog
        for turn in self.turns.iter() {
            formatted_dialog.push_str(&format!(
                "<start_of_turn>{:?}\n{}<end_of_turn>\n",
                turn.role,
                turn.content
            ));
        }

        formatted_dialog.push_str("<start_of_turn>model<end_of_turn>\n");

        formatted_dialog
    }
}
