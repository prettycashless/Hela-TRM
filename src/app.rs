use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Instant;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::boot::Boot;
use crate::system::SystemInfo;
use crate::tools;

const MAX_OUTPUT_LINES: usize = 4000;
const MAX_INPUT_CHARS: usize = 4096;

pub enum Task {
    Line(String),
    Done(Result<(), String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Boot,
    Home,
    Toolbox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
    Ok,
    Failed(String),
}

pub struct FieldSpec {
    pub label: &'static str,
    pub placeholder: &'static str,
    pub default: &'static str,
}

pub struct ToolSpec {
    pub group: &'static str,
    pub name: &'static str,
    pub tagline: &'static str,
    pub note: &'static str,
    pub fields: &'static [FieldSpec],
    pub run: fn(&[String], &Sender<Task>) -> Result<(), String>,
}

const F_TEXT: &[FieldSpec] = &[FieldSpec {
    label: "text",
    placeholder: "type anything",
    default: "the gate is open",
}];

const F_HASH: &[FieldSpec] = &[FieldSpec {
    label: "digest",
    placeholder: "paste a hex digest",
    default: "",
}];

const F_CAESAR: &[FieldSpec] = &[
    FieldSpec {
        label: "text",
        placeholder: "message",
        default: "the gate is open",
    },
    FieldSpec {
        label: "shift",
        placeholder: "0-25",
        default: "3",
    },
];

const F_JWT: &[FieldSpec] = &[FieldSpec {
    label: "token",
    placeholder: "header.payload.signature",
    default: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJoZWxhIiwibmFtZSI6InF1ZWVuIG9mIGhlbCIsImlhdCI6MTcwMDAwMDAwMH0.c2lnbmF0dXJl",
}];

const F_WORDLIST: &[FieldSpec] = &[
    FieldSpec {
        label: "pattern",
        placeholder: "seed word",
        default: "hela",
    },
    FieldSpec {
        label: "limit",
        placeholder: "max variants",
        default: "256",
    },
];

const F_PASSWORD: &[FieldSpec] = &[
    FieldSpec {
        label: "length",
        placeholder: "8-128",
        default: "20",
    },
    FieldSpec {
        label: "count",
        placeholder: "1-50",
        default: "5",
    },
];

const F_PORTSCAN: &[FieldSpec] = &[
    FieldSpec {
        label: "host",
        placeholder: "hostname or ip",
        default: "127.0.0.1",
    },
    FieldSpec {
        label: "ports",
        placeholder: "22,80,443 or 1-1024",
        default: "22,80,443,8080",
    },
    FieldSpec {
        label: "timeout",
        placeholder: "milliseconds",
        default: "300",
    },
];

const F_DNS: &[FieldSpec] = &[FieldSpec {
    label: "host",
    placeholder: "domain name",
    default: "github.com",
}];

const F_HTTP: &[FieldSpec] = &[FieldSpec {
    label: "url",
    placeholder: "https://example.com",
    default: "https://github.com",
}];

const F_FILE: &[FieldSpec] = &[FieldSpec {
    label: "path",
    placeholder: "/path/to/file",
    default: "",
}];

const NO_FIELDS: &[FieldSpec] = &[];

pub static TOOLS: &[ToolSpec] = &[
    ToolSpec {
        group: "crypto",
        name: "hash-forge",
        tagline: "four digests, one pass",
        note: "MD5 · SHA-1 · SHA-256 · SHA-512",
        fields: F_TEXT,
        run: tools::crypto::run_hash,
    },
    ToolSpec {
        group: "crypto",
        name: "hash-id",
        tagline: "name that digest",
        note: "length and fingerprint based identification",
        fields: F_HASH,
        run: tools::crypto::run_hash_id,
    },
    ToolSpec {
        group: "crypto",
        name: "base64",
        tagline: "weave between alphabets",
        note: "standard and URL-safe forms, decode both",
        fields: F_TEXT,
        run: tools::crypto::run_base64,
    },
    ToolSpec {
        group: "crypto",
        name: "caesar",
        tagline: "turn the alphabet",
        note: "encrypt with a shift, then brute-force 26",
        fields: F_CAESAR,
        run: tools::crypto::run_caesar,
    },
    ToolSpec {
        group: "crypto",
        name: "jwt-peel",
        tagline: "open the token",
        note: "decode header & payload; signatures are not verified",
        fields: F_JWT,
        run: tools::crypto::run_jwt,
    },
    ToolSpec {
        group: "crypto",
        name: "wordlist",
        tagline: "multiply a seed",
        note: "case permutations, capped by the limit field",
        fields: F_WORDLIST,
        run: tools::crypto::run_wordlist,
    },
    ToolSpec {
        group: "crypto",
        name: "password-forge",
        tagline: "shape a secret",
        note: "configurable length and count, mixed classes",
        fields: F_PASSWORD,
        run: tools::crypto::run_password,
    },
    ToolSpec {
        group: "network",
        name: "port-scan",
        tagline: "probe the gates",
        note: "TCP connect scan · only scan systems you own",
        fields: F_PORTSCAN,
        run: tools::net::run_port_scan,
    },
    ToolSpec {
        group: "network",
        name: "dns",
        tagline: "read the names",
        note: "resolve a host through the system resolver",
        fields: F_DNS,
        run: tools::net::run_dns,
    },
    ToolSpec {
        group: "network",
        name: "http-headers",
        tagline: "listen at the door",
        note: "HEAD request · status line and response headers",
        fields: F_HTTP,
        run: tools::net::run_http,
    },
    ToolSpec {
        group: "files",
        name: "autopsy",
        tagline: "dissect an artifact",
        note: "magic, entropy, hashes, hexdump and preview · 32 MiB sample",
        fields: F_FILE,
        run: tools::files::run_autopsy,
    },
    ToolSpec {
        group: "system",
        name: "sysinfo",
        tagline: "know the machine",
        note: "operator, OS, kernel, CPU, memory and network",
        fields: NO_FIELDS,
        run: tools::run_sysinfo,
    },
];

pub struct Session {
    pub inputs: Vec<String>,
    pub focus: usize,
    pub out: Vec<String>,
    pub status: Status,
    pub scroll: usize,
    pub started: Option<Instant>,
    rx: Option<Receiver<Task>>,
}

impl Session {
    fn from_tool(tool: &ToolSpec) -> Self {
        Self {
            inputs: tool
                .fields
                .iter()
                .map(|field| field.default.to_string())
                .collect(),
            focus: 0,
            out: Vec::new(),
            status: Status::Idle,
            scroll: 0,
            started: None,
            rx: None,
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub selected: usize,
    pub sessions: Vec<Session>,
    pub sys: SystemInfo,
    pub boot: Boot,
    pub should_quit: bool,
    pub started: Instant,
    pub home_entered: Instant,
    pub toolbox_entered: Instant,
    pub selection_changed: Instant,
    pub frame: u64,
}

impl App {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            screen: Screen::Boot,
            selected: 0,
            sessions: TOOLS.iter().map(Session::from_tool).collect(),
            sys: SystemInfo::collect(),
            boot: Boot::new(),
            should_quit: false,
            started: now,
            home_entered: now,
            toolbox_entered: now,
            selection_changed: now,
            frame: 0,
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.screen == Screen::Boot && self.boot.done() {
            self.enter_home();
        }
        self.drain_tasks();
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        if ctrl {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('c') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('u') => {
                    if self.screen == Screen::Toolbox {
                        self.clear_field();
                    }
                    return;
                }
                KeyCode::Char('w') => {
                    if self.screen == Screen::Toolbox {
                        self.delete_word();
                    }
                    return;
                }
                _ => {}
            }
        }

        match self.screen {
            Screen::Boot => {
                self.boot.skip();
                self.enter_home();
            }
            Screen::Home => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Tab | KeyCode::Down => {
                    self.enter_toolbox();
                }
                KeyCode::Char('j') => self.enter_toolbox(),
                KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            },
            Screen::Toolbox => self.toolbox_key(key),
        }
    }

    fn toolbox_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.enter_home(),
            KeyCode::Up => self.select_relative(-1),
            KeyCode::Down => self.select_relative(1),
            KeyCode::Tab => self.cycle_focus(1),
            KeyCode::BackTab => self.cycle_focus(-1),
            KeyCode::Enter => self.run_selected(),
            KeyCode::PageUp => self.scroll_output(8),
            KeyCode::PageDown => self.scroll_output(-8),
            KeyCode::Backspace => self.edit_backspace(),
            KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.edit_push(character)
            }
            _ => {}
        }
    }

    fn enter_home(&mut self) {
        self.screen = Screen::Home;
        self.home_entered = Instant::now();
    }

    fn enter_toolbox(&mut self) {
        self.screen = Screen::Toolbox;
        self.toolbox_entered = Instant::now();
        self.selection_changed = Instant::now();
    }

    fn select_relative(&mut self, delta: isize) {
        if TOOLS.is_empty() {
            return;
        }
        let count = TOOLS.len() as isize;
        let next = (self.selected as isize + delta).rem_euclid(count) as usize;
        if next != self.selected {
            self.selected = next;
            self.selection_changed = Instant::now();
        }
    }

    fn cycle_focus(&mut self, delta: isize) {
        let fields = TOOLS[self.selected].fields.len();
        if fields == 0 {
            return;
        }
        let session = &mut self.sessions[self.selected];
        session.focus = (session.focus as isize + delta).rem_euclid(fields as isize) as usize;
    }

    fn edit_push(&mut self, character: char) {
        let session = &mut self.sessions[self.selected];
        if let Some(input) = session.inputs.get_mut(session.focus) {
            if input.chars().count() < MAX_INPUT_CHARS {
                input.push(character);
            }
        }
    }

    fn edit_backspace(&mut self) {
        let session = &mut self.sessions[self.selected];
        if let Some(input) = session.inputs.get_mut(session.focus) {
            input.pop();
        }
    }

    fn clear_field(&mut self) {
        let session = &mut self.sessions[self.selected];
        if let Some(input) = session.inputs.get_mut(session.focus) {
            input.clear();
        }
    }

    fn delete_word(&mut self) {
        let session = &mut self.sessions[self.selected];
        if let Some(input) = session.inputs.get_mut(session.focus) {
            while input.ends_with(char::is_whitespace) {
                input.pop();
            }
            while let Some(last) = input.chars().last() {
                if last.is_whitespace() {
                    break;
                }
                input.pop();
            }
            while input.ends_with(char::is_whitespace) {
                input.pop();
            }
        }
    }

    fn scroll_output(&mut self, delta: isize) {
        let session = &mut self.sessions[self.selected];
        let maximum = session.out.len() as isize;
        session.scroll = (session.scroll as isize + delta).clamp(0, maximum) as usize;
    }

    fn run_selected(&mut self) {
        let index = self.selected;
        if self.sessions[index].status == Status::Running {
            return;
        }

        let run = TOOLS[index].run;
        let inputs = self.sessions[index].inputs.clone();
        let (sender, receiver) = mpsc::channel();

        {
            let session = &mut self.sessions[index];
            session.out.clear();
            session.status = Status::Running;
            session.started = Some(Instant::now());
            session.scroll = 0;
            session.rx = Some(receiver);
        }

        std::thread::spawn(move || {
            let result = run(&inputs, &sender);
            let _ = sender.send(Task::Done(result));
        });
    }

    fn drain_tasks(&mut self) {
        for session in &mut self.sessions {
            let mut messages = Vec::new();
            let mut disconnected = false;

            if let Some(receiver) = session.rx.as_ref() {
                loop {
                    match receiver.try_recv() {
                        Ok(task) => messages.push(task),
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            disconnected = true;
                            break;
                        }
                    }
                }
            }

            let mut finished = false;
            for task in messages {
                match task {
                    Task::Line(line) => session.out.push(line),
                    Task::Done(result) => {
                        finished = true;
                        session.rx = None;
                        session.started = None;
                        session.status = match result {
                            Ok(()) => Status::Ok,
                            Err(error) => Status::Failed(error),
                        };
                    }
                }
            }

            if disconnected && !finished {
                let was_running = session.status == Status::Running;
                session.rx = None;
                if was_running {
                    session.started = None;
                    session.status = Status::Failed("worker stopped unexpectedly".into());
                }
            }

            if session.out.len() > MAX_OUTPUT_LINES {
                let excess = session.out.len() - MAX_OUTPUT_LINES;
                session.out.drain(0..excess);
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.sessions
            .iter()
            .any(|session| session.status == Status::Running)
    }

    pub fn pulse(&self) -> f32 {
        ((self.started.elapsed().as_secs_f32() * 2.4).sin() + 1.0) * 0.5
    }

    pub fn home_progress(&self) -> f32 {
        (self.home_entered.elapsed().as_secs_f32() / 0.95).clamp(0.0, 1.0)
    }

    pub fn toolbox_progress(&self) -> f32 {
        (self.toolbox_entered.elapsed().as_secs_f32() / 0.55).clamp(0.0, 1.0)
    }

    pub fn selection_flash(&self) -> f32 {
        (1.0 - self.selection_changed.elapsed().as_secs_f32() / 0.35).clamp(0.0, 1.0)
    }

    pub fn spinner(&self) -> &'static str {
        const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        FRAMES[(self.frame / 2) as usize % FRAMES.len()]
    }

    pub fn scan_row(&self, rows: usize) -> usize {
        if rows == 0 {
            return 0;
        }
        (self.started.elapsed().as_secs_f32() * 9.0) as usize % rows
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(character: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(character), KeyModifiers::CONTROL)
    }

    fn wait_for_status(app: &mut App, index: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.sessions[index].status == Status::Running && Instant::now() < deadline {
            app.tick();
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn screens_advance_with_keys() {
        let mut app = App::new();
        assert_eq!(app.screen, Screen::Boot);

        app.on_key(key(KeyCode::Char('x')));
        assert_eq!(app.screen, Screen::Home);

        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Toolbox);

        app.on_key(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Home);

        app.on_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn field_editing_and_focus() {
        let mut app = App::new();
        app.screen = Screen::Toolbox;
        app.selected = 5;

        app.on_key(ctrl('u'));
        assert_eq!(app.sessions[5].inputs[0], "");

        for character in "hela".chars() {
            app.on_key(key(KeyCode::Char(character)));
        }
        assert_eq!(app.sessions[5].inputs[0], "hela");

        app.on_key(key(KeyCode::Tab));
        assert_eq!(app.sessions[5].focus, 1);
        app.on_key(key(KeyCode::Tab));
        assert_eq!(app.sessions[5].focus, 0);
        app.on_key(key(KeyCode::BackTab));
        assert_eq!(app.sessions[5].focus, 1);

        app.on_key(ctrl('u'));
        assert_eq!(app.sessions[5].inputs[1], "");
        app.on_key(key(KeyCode::Backspace));
        assert_eq!(app.sessions[5].inputs[1], "");
    }

    #[test]
    fn previous_word_is_deleted() {
        let mut app = App::new();
        app.screen = Screen::Toolbox;
        app.selected = 0;
        app.sessions[0].inputs[0] = "the gate is open".into();

        app.on_key(ctrl('w'));
        assert_eq!(app.sessions[0].inputs[0], "the gate is");
    }

    #[test]
    fn hash_tool_runs_and_streams_output() {
        let mut app = App::new();
        app.screen = Screen::Toolbox;
        app.selected = 0;
        app.sessions[0].inputs[0] = "hela".into();

        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.sessions[0].status, Status::Running);

        wait_for_status(&mut app, 0);
        assert!(matches!(app.sessions[0].status, Status::Ok));

        let output = app.sessions[0].out.join("\n");
        assert!(output.contains("⟡ hash forge"));
        assert!(output.contains("sha256"));
        assert!(output.contains("input"));
    }

    #[test]
    fn failing_tool_reports_an_error() {
        let mut app = App::new();
        app.screen = Screen::Toolbox;
        app.selected = 0;
        app.sessions[0].inputs[0].clear();

        app.on_key(key(KeyCode::Enter));
        wait_for_status(&mut app, 0);

        assert!(matches!(app.sessions[0].status, Status::Failed(_)));
    }
}
