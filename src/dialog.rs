#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Role {
    Model,
    User,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Turn {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Dialog {
    thread: String,
    turns: Vec<Turn>,
}

impl Dialog {
    pub fn new(thread: String) -> Dialog {
        Dialog {
            thread,
            turns: Vec::new(),
        }
    }

    pub fn add(&mut self, role: Role, content: String) {
        self.turns.push(Turn { role, content });
    }

    pub fn len(&self) -> usize {
        self.turns.len()
    }

    // Keeps the first message and the most recent 'count' turns
    pub fn truncate_history(&mut self, count: usize) {
        if self.turns.len() <= count {
            return;
        }

        // Always preserve the first message
        let mut preserved = Vec::new();

        if !self.turns.is_empty() {
            preserved.push(self.turns[0].clone());
        }

        // Keep the most recent 'count' turns
        let start_idx = self.turns.len().saturating_sub(count);
        preserved.extend(self.turns[start_idx..].to_vec());

        self.turns = preserved;
    }

    pub fn format(&self) -> String {
        let mut formatted_dialog = String::new();

        for turn in self.turns.iter() {
            let role_str = match turn.role {
                Role::User => "user",
                Role::Model => "model",
            };

            formatted_dialog.push_str(&format!(
                "<start_of_turn>{}\n{}<end_of_turn>\n",
                role_str, turn.content
            ));
        }

        // Add final model turn to prompt the model to respond
        formatted_dialog.push_str("<start_of_turn>model\n");

        formatted_dialog
    }

    pub fn thread(&self) -> &str {
        &self.thread
    }

    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }

    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    // Format dialog using a custom formatting function
    pub fn format_with<F>(&self, format_fn: F) -> String
    where
        F: Fn(&Role, &str) -> String,
    {
        let mut result = String::new();

        for turn in &self.turns {
            result.push_str(&format_fn(&turn.role, &turn.content));
        }

        result
    }
}
