use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style::{Color as CrosstermColor, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::color::{ColorRef, coloredText, strip_ansi_codes, visible_length};

// Color theme for the prompt
#[derive(Clone)]
pub struct ColorTheme<'a> {
    pub prompt_color: ColorRef<'a>,
    pub input_color: ColorRef<'a>,
    pub suggestion_color: ColorRef<'a>,
    pub selected_suggestion_color: ColorThemeSelectedSuggestion<'a>,
    pub hint_color: ColorRef<'a>,
}

#[derive(Clone)]
pub struct ColorThemeSelectedSuggestion<'a> {
    pub fg: ColorRef<'a>,
    pub bg: ColorRef<'a>,
}

impl<'a> ColorTheme<'a> {
    pub fn default() -> Self {
        ColorTheme {
            prompt_color: ColorRef::Named("default", "cyan"),
            input_color: ColorRef::Named("default", "white"),
            suggestion_color: ColorRef::Named("default", "white"),
            selected_suggestion_color: ColorThemeSelectedSuggestion {
                fg: ColorRef::Named("default", "yellow"),
                bg: ColorRef::Named("default", "dark_gray"),
            },
            hint_color: ColorRef::Named("default", "gray"),
        }
    }

    pub fn dark() -> Self {
        ColorTheme {
            prompt_color: ColorRef::Named("default", "light_cyan"),
            input_color: ColorRef::Named("default", "light_gray"),
            suggestion_color: ColorRef::Named("default", "light_gray"),
            selected_suggestion_color: ColorThemeSelectedSuggestion {
                fg: ColorRef::Named("default", "light_yellow"),
                bg: ColorRef::Named("default", "dark_gray"),
            },
            hint_color: ColorRef::Named("default", "gray"),
        }
    }

    pub fn vibrant() -> Self {
        ColorTheme {
            prompt_color: ColorRef::Named("default", "magenta"),
            input_color: ColorRef::Named("default", "white"),
            suggestion_color: ColorRef::Named("default", "white"),
            selected_suggestion_color: ColorThemeSelectedSuggestion {
                fg: ColorRef::Named("default", "light_magenta"),
                bg: ColorRef::Named("default", "dark_magenta"),
            },
            hint_color: ColorRef::Named("default", "light_gray"),
        }
    }
}

// Command argument definition
#[derive(Debug, Clone)]
pub struct CommandArg {
    pub name: String,
    pub arg_type: String, // e.g., "int", "string"
    pub range: Option<(i32, i32)>, // For int: (min, max)
    pub optional: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<CommandArg>,
    pub subcommands: Vec<Command>,
    pub handler: Option<fn(HashMap<String, String>) -> String>, // Function to handle command
}

#[derive(Debug, Clone)]
pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry { commands: vec![] }
    }

    pub fn register_command(&mut self, command: Command) {
        // Ensure optional args are at the end
        let mut required = vec![];
        let mut optional = vec![];
        for arg in command.args.iter() {
            if arg.optional {
                optional.push(arg.clone());
            } else {
                required.push(arg.clone());
            }
        }
        let mut new_command = command.clone();
        new_command.args = required.into_iter().chain(optional).collect();
        self.commands.push(new_command);

        // Register subcommands recursively
        for subcommand in command.subcommands {
            self.register_command(subcommand);
        }
    }

    pub fn find_command(&self, name: &str) -> Option<&Command> {
        let parts: Vec<&str> = name.split_whitespace().collect();
        let mut current = self.commands.iter().find(|c| c.name == parts[0])?;
        for part in parts.iter().skip(1) {
            current = current.subcommands.iter().find(|c| c.name == *part)?;
        }
        Some(current)
    }

    pub fn get_suggestions(&self, input: &str) -> (Vec<String>, String) {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        let mut suggestions = vec![];
        let mut hint = String::new();

        if parts.is_empty() {
            suggestions = self.commands.iter().map(|c| c.name.clone()).collect();
            return (suggestions, hint);
        }

        let command_name = parts[0];
        if parts.len() == 1 {
            suggestions = self
                .commands
                .iter()
                .filter(|c| c.name.starts_with(command_name))
                .map(|c| c.name.clone())
                .collect();
            return (suggestions, hint);
        }

        // Find the command up to the last completed part
        let command_path = parts[..parts.len() - 1].join(" ");
        if let Some(command) = self.find_command(&command_path) {
            let last_part = parts.last().unwrap();
            if last_part.contains(':') {
                // Named argument input, suggest values
                suggestions = vec![];
                let arg_index = parts.iter().skip(parts.len().min(1)).filter(|p| !p.contains(':')).count();
                if arg_index < command.args.len() {
                    let arg = &command.args[arg_index];
                    hint = format!("<{}:{}", arg.name, arg.arg_type);
                    if let Some((min, max)) = arg.range {
                        hint.push_str(&format!(" {{{}..{}}}", min, max));
                    }
                    hint.push('>');
                    if arg.optional {
                        hint.push_str(&format!("?{}", arg.default.as_ref().unwrap_or(&"none".to_string())));
                    }
                    if arg.arg_type == "int" {
                        suggestions = vec!["0", "1", "10", "100"]
                            .into_iter()
                            .map(String::from)
                            .filter(|s| s.starts_with(last_part))
                            .collect();
                    } else if let Some(default) = &arg.default {
                        suggestions = vec![default.clone()]
                            .into_iter()
                            .filter(|s| s.starts_with(last_part))
                            .collect();
                    }
                }
            } else {
                // Suggest subcommands or arguments
                suggestions = command
                    .subcommands
                    .iter()
                    .filter(|c| c.name.starts_with(last_part))
                    .map(|c| format!("{} {}", command_path, c.name).trim().to_string())
                    .collect();
                let arg_index = parts.iter().skip(parts.len().min(1)).filter(|p| !p.contains(':')).count();
                if arg_index < command.args.len() {
                    let arg = &command.args[arg_index];
                    hint = format!("<{}:{}", arg.name, arg.arg_type);
                    if let Some((min, max)) = arg.range {
                        hint.push_str(&format!(" {{{}..{}}}", min, max));
                    }
                    hint.push('>');
                    if arg.optional {
                        hint.push_str(&format!("?{}", arg.default.as_ref().unwrap_or(&"none".to_string())));
                    }
                    suggestions.push(format!("{} {}:", command_path, arg.name).trim().to_string());
                }
            }
        }

        (suggestions, hint)
    }

    pub fn execute_command(&self, input: &str) -> Option<String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        // Find the deepest command
        let mut command = None;
        let mut command_len = 0;
        for i in 1..=parts.len() {
            if let Some(cmd) = self.find_command(&parts[..i].join(" ")) {
                command = Some(cmd);
                command_len = i;
            } else {
                break;
            }
        }

        let command = command?;
        let mut args = HashMap::new();
        let mut named_args = HashMap::new();

        // Parse arguments (named or positional)
        let mut positional_args = vec![];
        for part in parts.iter().skip(command_len) {
            if let Some((key, value)) = part.split_once(':') {
                named_args.insert(key.to_string(), value.to_string());
            } else {
                positional_args.push(part.to_string());
            }
        }

        // Assign positional arguments
        for (i, arg) in command.args.iter().enumerate() {
            if i < positional_args.len() {
                args.insert(arg.name.clone(), positional_args[i].clone());
            } else if let Some(value) = named_args.get(&arg.name) {
                args.insert(arg.name.clone(), value.clone());
            } else if let Some(default) = &arg.default {
                args.insert(arg.name.clone(), default.clone());
            } else if !arg.optional {
                return Some(format!("Missing required argument: {}", arg.name));
            }
        }

        command.handler.map(|f| f(args))
    }
}

// Progress bar configuration
pub struct ProgressBar {
    total: u64,
    current: u64,
    width: usize,
    symbol: char,
    color_ref: ColorRef<'static>,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        ProgressBar {
            total,
            current: 0,
            width: 50,
            symbol: '█',
            color_ref: ColorRef::Named("default", "blue"),
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_symbol(mut self, symbol: char) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_color(mut self, color_ref: ColorRef<'static>) -> Self {
        self.color_ref = color_ref;
        self
    }

    pub fn advance(&mut self, delta: u64) {
        self.current = (self.current + delta).min(self.total);
        self.render();
    }

    pub fn render(&self) {
        let progress = self.current as f64 / self.total as f64;
        let filled = (self.width as f64 * progress) as usize;
        let bar: String = std::iter::repeat(self.symbol)
            .take(filled)
            .chain(std::iter::repeat(' ').take(self.width - filled))
            .collect();
        let percentage = (progress * 100.0) as u32;
        let text = format!("[{}] {}%", bar, percentage);
        if let Ok(colored) = coloredText(&text, &self.color_ref) {
            print!("\r{}", colored);
            io::stdout().flush().unwrap();
        }
    }

    pub fn finish(&self) {
        println!();
    }
}

// Prompt configuration
#[derive(Clone)]
pub struct PromptConfig<'a> {
    prompt: &'a str,
    registry: CommandRegistry,
    history: Vec<String>,
    max_history: usize,
    theme: ColorTheme<'a>,
    max_suggestions: usize,
}

impl<'a> PromptConfig<'a> {
    pub fn new(prompt: &'a str, registry: CommandRegistry) -> Self {
        PromptConfig {
            prompt,
            registry,
            history: vec![],
            max_history: 50,
            theme: ColorTheme::default(),
            max_suggestions: 5,
        }
    }

    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.history = history;
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    pub fn with_theme(mut self, theme: ColorTheme<'a>) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }
}

// Interactive prompt
pub struct InteractivePrompt<'a> {
    config: PromptConfig<'a>,
    input: String,
    cursor_pos: usize,
    history_index: Option<usize>,
    suggestions: Vec<String>,
    selected_suggestion: Option<usize>,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    running: bool,
    hint: String,
}

impl<'a> InteractivePrompt<'a> {
    pub fn new(config: PromptConfig<'a>) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(InteractivePrompt {
            config,
            input: String::new(),
            cursor_pos: 0,
            history_index: None,
            suggestions: vec![],
            selected_suggestion: None,
            terminal,
            running: true,
            hint: String::new(),
        })
    }

    fn update_suggestions(&mut self) {
        let (suggestions, hint) = self.config.registry.get_suggestions(&self.input);
        self.suggestions = suggestions;
        self.hint = hint;
        self.selected_suggestion = if self.suggestions.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn render(&mut self) -> io::Result<()> {
        let config = self.config.clone();
        let input = self.input.clone();
        let suggestions = self.suggestions.clone();
        let selected_suggestion = self.selected_suggestion;
        let hint = self.hint.clone();
        let prompt_len = visible_length(config.prompt);
        let input_len = visible_length(&input);
        let terminal_width = self.terminal.size()?.width as usize;
        let total_len = prompt_len + input_len;
        let padding = if total_len < terminal_width {
            (terminal_width - total_len) / 2
        } else {
            0
        };

        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(config.max_suggestions as u16 + 2),
                    Constraint::Length(1),
                ])
                .split(f.area());

            // Render prompt and input (centered)
            let prompt_text = coloredText(config.prompt, &config.theme.prompt_color).unwrap_or_else(|_| config.prompt.to_string());
            let input_text = coloredText(&input, &config.theme.input_color).unwrap_or_else(|_| input.clone());
            let combined_text = format!("{}{}", prompt_text, input_text);
            let paragraph = Paragraph::new(combined_text)
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Center);
            f.render_widget(paragraph, chunks[0]);

            // Render suggestions dropdown
            let items: Vec<ListItem> = suggestions
                .iter()
                .take(config.max_suggestions)
                .enumerate()
                .map(|(i, s)| {
                    let style = if selected_suggestion == Some(i) {
                        Style::default()
                            .fg(config.theme.selected_suggestion_color.fg.resolve().map(|c| Color::Rgb(c.r, c.g, c.b)).unwrap_or(Color::Yellow))
                            .bg(config.theme.selected_suggestion_color.bg.resolve().map(|c| Color::Rgb(c.r, c.g, c.b)).unwrap_or(Color::DarkGray))
                    } else {
                        Style::default()
                            .fg(config.theme.suggestion_color.resolve().map(|c| Color::Rgb(c.r, c.g, c.b)).unwrap_or(Color::White))
                    };
                    ListItem::new(s.clone()).style(style)
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Suggestions"));
            let mut list_state = ListState::default();
            list_state.select(selected_suggestion);
            f.render_stateful_widget(list, chunks[1], &mut list_state);

            // Render hint
            let hint_text = coloredText(&hint, &config.theme.hint_color).unwrap_or_else(|_| hint.clone());
            let hint_paragraph = Paragraph::new(hint_text)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(hint_paragraph, chunks[2]);

            // Set cursor position (adjusted for centering)
            let cursor_x = (padding + prompt_len + self.cursor_pos) as u16;
            f.set_cursor_position((cursor_x, chunks[0].y));
        })?;
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> io::Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Enter, _) => {
                if self.input.trim() == "exit" {
                    self.running = false;
                    return Ok(());
                }
                if !self.input.is_empty() {
                    self.config.history.push(self.input.clone());
                    if self.config.history.len() > self.config.max_history {
                        self.config.history.remove(0);
                    }
                    if let Some(result) = self.config.registry.execute_command(&self.input) {
                        let colored_result = coloredText(
                            &format!("Result: {}", result),
                            &ColorRef::Named("default", "yellow"),
                        ).unwrap_or_else(|_| format!("Result: {}", result));
                        println!("\n{}", colored_result);
                        io::stdout().flush()?;
                    }
                    self.input.clear();
                    self.cursor_pos = 0;
                    self.history_index = None;
                    self.update_suggestions();
                }
            }
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.update_suggestions();
            }
            (KeyCode::Backspace, _) => {
                if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    self.update_suggestions();
                }
            }
            (KeyCode::Left, _) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            (KeyCode::Right, _) => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }
            (KeyCode::Up, _) => {
                if !self.suggestions.is_empty() {
                    self.selected_suggestion = Some(
                        self.selected_suggestion
                            .map_or(0, |i| if i == 0 { 0 } else { i - 1 }),
                    );
                } else if !self.config.history.is_empty() {
                    let max_index = self.config.history.len() - 1;
                    self.history_index = Some(
                        self.history_index
                            .map_or(max_index, |i| if i == 0 { 0 } else { i - 1 }),
                    );
                    self.input = self.config.history[self.history_index.unwrap()].clone();
                    self.cursor_pos = self.input.len();
                    self.update_suggestions();
                }
            }
            (KeyCode::Down, _) => {
                if !self.suggestions.is_empty() {
                    self.selected_suggestion = Some(
                        self.selected_suggestion.map_or(0, |i| {
                            if i + 1 < self.suggestions.len().min(self.config.max_suggestions) {
                                i + 1
                            } else {
                                i
                            }
                        }),
                    );
                } else if !self.config.history.is_empty() {
                    self.history_index = Some(
                        self.history_index.map_or(0, |i| {
                            if i + 1 < self.config.history.len() {
                                i + 1
                            } else {
                                i
                            }
                        }),
                    );
                    self.input = self.config.history[self.history_index.unwrap()].clone();
                    self.cursor_pos = self.input.len();
                    self.update_suggestions();
                }
            }
            (KeyCode::Tab, _) => {
                if let Some(idx) = self.selected_suggestion {
                    if idx < self.suggestions.len() {
                        let suggestion = &self.suggestions[idx];
                        let parts: Vec<&str> = self.input.trim().split_whitespace().collect();
                        if parts.is_empty() {
                            self.input = suggestion.clone();
                        } else if parts.len() > 1 && !parts.last().unwrap().contains(':') {
                            let last_space = self.input.rfind(' ').unwrap_or(0);
                            self.input = format!("{}{}", &self.input[..last_space], suggestion);
                        } else {
                            self.input = suggestion.clone();
                        }
                        self.cursor_pos = self.input.len();
                        self.update_suggestions();
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn run(mut self) -> io::Result<()> {
        execute!(
            self.terminal.backend_mut(),
            terminal::EnterAlternateScreen,
            cursor::EnableBlinking,
            cursor::Show
        )?;
        self.update_suggestions();
        while self.running {
            self.render()?;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key)?;
                }
            }
        }
        execute!(
            self.terminal.backend_mut(),
            terminal::LeaveAlternateScreen,
            cursor::Show
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

// Main prompt function
pub fn prompt(config: PromptConfig) -> io::Result<()> {
    let prompt = InteractivePrompt::new(config)?;
    prompt.run()
}

// Simple print with color
pub fn print_colored(text: &str, color_ref: &ColorRef) -> io::Result<()> {
    let colored = coloredText(text, color_ref).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    print!("{}", colored);
    io::stdout().flush()
}

// Helper trait to resolve ColorRef to ratatui Color
trait ColorRefExt<'a> {
    fn resolve(&self) -> Option<crate::color::Color>;
}

impl<'a> ColorRefExt<'a> for ColorRef<'a> {
    fn resolve(&self) -> Option<crate::color::Color> {
        crate::color::resolve_color_ref(self)
    }
}