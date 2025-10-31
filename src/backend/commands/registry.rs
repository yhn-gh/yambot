use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Permission level required to execute a command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandPermission {
    /// Anyone can use this command
    Everyone,
    /// Only subscribers can use this command
    Subscriber,
    /// Only VIPs can use this command
    Vip,
    /// Only moderators can use this command
    Moderator,
    /// Only the broadcaster can use this command
    Broadcaster,
}

impl CommandPermission {
    /// Check if user badges meet the permission requirement
    /// Implements a permission hierarchy: Broadcaster > Moderator > VIP > Subscriber > Everyone
    pub fn has_permission(&self, badges: &[crate::backend::twitch::Badge]) -> bool {
        // Check if user is broadcaster (has all permissions)
        let is_broadcaster = badges.iter().any(|b| b.set_id == "broadcaster");
        if is_broadcaster {
            return true;
        }

        // Check if user is moderator (has all permissions except broadcaster-only)
        let is_moderator = badges.iter().any(|b| b.set_id == "moderator");
        if is_moderator && !matches!(self, CommandPermission::Broadcaster) {
            return true;
        }

        // Check specific permission level
        match self {
            CommandPermission::Everyone => true,
            CommandPermission::Subscriber => {
                badges.iter().any(|b| b.set_id == "subscriber" || b.set_id == "founder")
            }
            CommandPermission::Vip => badges.iter().any(|b| b.set_id == "vip"),
            CommandPermission::Moderator => false, // Already checked above
            CommandPermission::Broadcaster => false, // Already checked above
        }
    }
}

/// Action to perform when a command is executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
    /// Play a sound effect
    PlaySound { sound_name: String },
    /// Use text-to-speech
    TextToSpeech { message: String },
    /// Send a message to chat
    SendMessage { message: String },
    /// Reply to the user who sent the command
    Reply { message: String },
    /// Multiple actions in sequence
    Multiple { actions: Vec<CommandAction> },
    // Future actions can be added here:
    // Ban, Timeout, RunScript, etc.
}

/// A command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// The command trigger (without the ! prefix)
    pub trigger: String,
    /// Description of what the command does
    pub description: String,
    /// Permission level required
    pub permission: CommandPermission,
    /// What the command does
    pub action: CommandAction,
    /// Cooldown in seconds (0 = no cooldown)
    pub cooldown: u64,
    /// Whether the command is enabled
    pub enabled: bool,
}

impl Command {
    /// Create a new command
    pub fn new(
        trigger: String,
        description: String,
        permission: CommandPermission,
        action: CommandAction,
    ) -> Self {
        Self {
            trigger,
            description,
            permission,
            action,
            cooldown: 0,
            enabled: true,
        }
    }

    /// Builder method to set cooldown
    pub fn with_cooldown(mut self, cooldown: u64) -> Self {
        self.cooldown = cooldown;
        self
    }

    /// Builder method to set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Registry for managing commands
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    #[serde(skip)]
    last_executed: HashMap<String, std::time::Instant>,
}

impl CommandRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command
    pub fn register(&mut self, command: Command) {
        let trigger = command.trigger.clone();

        // If this is an update (command already exists), clear its cooldown state
        // This ensures cooldown changes take effect immediately
        if self.commands.contains_key(&trigger) {
            self.last_executed.remove(&trigger);
        }

        self.commands.insert(trigger, command);
    }

    /// Unregister a command
    pub fn unregister(&mut self, trigger: &str) -> Option<Command> {
        // Also remove cooldown state when unregistering
        self.last_executed.remove(trigger);
        self.commands.remove(trigger)
    }

    /// Get a command by trigger
    pub fn get(&self, trigger: &str) -> Option<&Command> {
        self.commands.get(trigger)
    }

    /// Get a mutable reference to a command by trigger
    pub fn get_mut(&mut self, trigger: &str) -> Option<&mut Command> {
        self.commands.get_mut(trigger)
    }

    /// List all commands
    pub fn list(&self) -> Vec<&Command> {
        self.commands.values().collect()
    }

    /// Check if a command is on cooldown
    pub fn is_on_cooldown(&self, trigger: &str) -> bool {
        if let Some(command) = self.get(trigger) {
            if command.cooldown == 0 {
                return false;
            }

            if let Some(last_time) = self.last_executed.get(trigger) {
                let elapsed = last_time.elapsed().as_secs();
                return elapsed < command.cooldown;
            }
        }
        false
    }

    /// Get remaining cooldown time in seconds
    pub fn remaining_cooldown(&self, trigger: &str) -> Option<u64> {
        if let Some(command) = self.get(trigger) {
            if command.cooldown == 0 {
                return None;
            }

            if let Some(last_time) = self.last_executed.get(trigger) {
                let elapsed = last_time.elapsed().as_secs();
                if elapsed < command.cooldown {
                    return Some(command.cooldown - elapsed);
                }
            }
        }
        None
    }

    /// Update the last execution time for a command
    pub fn update_cooldown(&mut self, trigger: &str) {
        self.last_executed
            .insert(trigger.to_string(), std::time::Instant::now());
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
        self.last_executed.clear();
    }

    /// Get the number of registered commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}
