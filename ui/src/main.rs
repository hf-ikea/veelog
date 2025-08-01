use hamlib::{
    lock::{self, Hamlib},
    rig::Rig,
    sys::RIG_MODEL_IC7200,
    token::TOK_PATHNAME,
    types::VFO,
};
use iced::{alignment::Horizontal, event::{self, Status}, keyboard::{key::Named, Key, Modifiers}, widget::{self, button, column, container, row, scrollable, text_input, Column}, window, Element, Length, Task, Theme
};
use log::error;
use std::{collections::HashMap, env, fs::remove_dir_all, path::Path, time::Duration};

use db::data::{FieldType, Log, LogHeader};

#[derive(Debug, Clone, Copy)]
pub enum Screen {
    Entry,
    LogList,
}

#[derive(Debug, Clone)]
pub enum Message {
    EntrySelected,
    ContentChanged((FieldType, String)),
    KeyPressed(String),
    InitLog,
    ImportADIF,
    InitHamlib,
    OpenRig,
    UpdateRig,
}

pub struct RigState {
    rig: Option<Rig>,
    freq: f64,
    mode: u64,
    width: i64,
}

pub struct State {
    hamlib: Option<Hamlib>,
    rig_state: RigState,
    cur_log: Option<Log>,
    screen: Screen,
    content: HashMap<FieldType, String>,
    focused_entry: usize,
    entry_fields: Vec<FieldType>,
}

impl Default for State {
    fn default() -> Self {
        let entry_fields = vec![
            FieldType::WorkedCall,
            FieldType::SentRST,
            FieldType::RcvdRST,
        ];
        Self {
            hamlib: None,
            rig_state: RigState {
                rig: None,
                freq: 0.0,
                mode: 0,
                width: 0,
            },
            cur_log: None,
            screen: Screen::LogList,
            content: HashMap::new(),
            focused_entry: 0,
            entry_fields,
        }
    }
}

impl State {
    pub fn title(&self) -> String {
        format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EntrySelected => self.screen = Screen::Entry,
            Message::InitLog => {
                let path = env::temp_dir().join(Path::new("veelog-tests-db"));
                let _ = remove_dir_all(&path);
                let header = LogHeader::new("N0CALL", "");
                self.cur_log = Some(Log::new_from_path(&path, header).unwrap());
            }
            Message::ImportADIF => {
                if let Some(log) = &mut self.cur_log {
                    log.import_adif_file("testlog2.adi".into()).unwrap();
                }
            }
            Message::InitHamlib => {
                let lib = Hamlib::new().unwrap();
                unsafe { lock::Hamlib::init_hamlib() };
                lock::set_log_level(&lib, hamlib::LogLevel::Trace);
                lock::set_log_timestamps(&lib, true);
                lock::load_rig_backends(&lib).unwrap();
                //params::init_params(lib);
                self.hamlib = Some(lib);
            }
            Message::OpenRig => {
                if self.rig_state.rig.is_some() {
                    return Task::none();
                }
                if let Some(lib) = &self.hamlib {
                    let mut my_rig = Rig::new(lib, RIG_MODEL_IC7200).unwrap();
                    my_rig.set_conf(lib, TOK_PATHNAME, c"/dev/serial/by-id/usb-Silicon_Labs_CP2102_USB_to_UART_Bridge_Controller_IC-7200_0202084-if00-port0").unwrap();
                    my_rig.open(lib).unwrap();
                    self.rig_state.rig = Some(my_rig)
                }
            }
            Message::UpdateRig => {
                if let Some(lib) = &self.hamlib {
                    if let Some(rig) = &self.rig_state.rig {
                        self.rig_state.freq = rig.get_freq(&lib, VFO::RIG_VFO_CURR).unwrap();
                        let (m, w) = rig.get_mode(&lib, VFO::RIG_VFO_CURR).unwrap();
                        self.rig_state.mode = m;
                        self.rig_state.width = w;
                    }
                }
            }
            Message::ContentChanged((k, v)) => {
                let mut v = v;
                match k {
                    FieldType::WorkedCall => {
                        if !v.chars().all(char::is_alphanumeric) {
                            return Task::none();
                        }
                        v.truncate(15);
                        v.make_ascii_uppercase()
                    }
                    FieldType::SentRST => {
                        if v.parse::<u32>().is_err() && v != "" {
                            return Task::none();
                        }
                        v.truncate(3);
                    }
                    FieldType::RcvdRST => {
                        if v.parse::<u32>().is_err() && v != "" {
                            return Task::none();
                        }
                        v.truncate(3);
                    }
                    FieldType::GridSquare => todo!(),
                    FieldType::PrimaryAdminSubdiv => todo!(),
                    FieldType::SentSerial => {
                        if v.parse::<u32>().is_err() && v != "" {
                            return Task::none();
                        }
                    }
                    FieldType::RcvdSerial => {
                        if v.parse::<u32>().is_err() && v != "" {
                            return Task::none();
                        }
                    }
                    _ => todo!(),
                };
                *self.content.entry(k).or_insert("".to_string()) = v.to_string();
            }
            Message::KeyPressed(key) => {
                match key.as_str() {
                    "Tab" => {
                        self.focused_entry += 1;
                        if self.focused_entry >= self.entry_fields.len() {
                            self.focused_entry = 0;
                        }
                        return text_input::focus(self.focused_entry.to_string());
                    }
                    "TabShift" => {
                        if self.focused_entry as isize - 1 < 0 {
                            self.focused_entry = 0;
                        } else {
                            self.focused_entry -= 1;
                        }
                        return text_input::focus(self.focused_entry.to_string());
                    }
                    _ => return Task::none(),
                };
            }
        };
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let controls = row![].push(button("Entry").on_press(Message::EntrySelected));
        let screen = match self.screen {
            Screen::Entry => self.entry(),
            Screen::LogList => self.log_list(),
        };
        let info = row![widget::text(format!(
            "rig freq: {:.2}kHz, mode: {}, width: {}",
            self.rig_state.freq / 1e3,
            self.rig_state.mode,
            self.rig_state.width
        ))];

        let content = column![controls, info, screen,];

        match self.screen {
            Screen::Entry => content.into(),
            Screen::LogList => container(scrollable(container(content))).into(),
        }
    }

    pub fn entry(&self) -> Element<'_, Message> {
        let mut row = row![].spacing(10);
        let mut i = 0;
        for f in &self.entry_fields {
            let width = match f {
                FieldType::WorkedCall => 230,
                FieldType::SentRST => 100,
                FieldType::RcvdRST => 100,
                _ => 300,
            };
            let placeholder = match f {
                FieldType::SentRST => "59",
                FieldType::RcvdRST => "59",
                _ => "",
            };
            let col = column![].push(widget::text(f.to_string())).push(
                text_input(
                    "",
                    self.content.get(&f).get_or_insert(&placeholder.to_string()),
                )
                .id(i.to_string())
                .on_input(move |v| Message::ContentChanged((f.clone(), v)))
                .align_x(Horizontal::Right)
                .size(42)
                .width(width),
            );
            i += 1;
            row = row.push(col);
        }

        container(row).center_x(Length::Fill).into()
    }

    pub fn log_list(&self) -> Element<'_, Message> {
        let disp_fields = vec![
            FieldType::Timestamp,
            FieldType::WorkedCall,
            FieldType::Frequency,
            FieldType::Mode,
            FieldType::SentRST,
            FieldType::RcvdRST,
        ];
        // Vec of "Columns"
        let mut table = Vec::with_capacity(disp_fields.len());
        for f in &disp_fields {
            table.push(vec![widget::text(f.to_string()).into()]);
        }
        if let Some(log) = &self.cur_log {
            for record in log.get_records() {
                for (i, ty) in disp_fields.iter().enumerate() {
                    match record.get_field(ty) {
                        Some(v) => table[i].push(widget::text(v.to_string()).into()),
                        None => table[i].push(widget::text("").into()),
                    }
                }
            }
        }
        let buttons = row![
            button("Init new Log").on_press(Message::InitLog),
            button("Import ADIF").on_press(Message::ImportADIF),
            button("Init hamlib").on_press(Message::InitHamlib),
            button("Open rig").on_press(Message::OpenRig)
        ];
        let mut row = row![].spacing(10).width(Length::Fill);
        for x in table {
            let y = Column::from_vec(x);
            row = row.push(y);
        }
        column![buttons, row,].into()
    }

    fn rig_update_timer(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(700)).map(|_| Message::UpdateRig)
    }

    fn keyboard_listener(&self) -> iced::Subscription<Message> {
        event::listen_with(|event, status, _| match (event, status) {
            (
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: Key::Named(Named::Tab),
                    modifiers: Modifiers::SHIFT,
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::KeyPressed("TabShift".into())),
            (
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: Key::Named(Named::Tab),
                    ..
                }),
                Status::Ignored,
            ) => Some(Message::KeyPressed("Tab".into())),
            _ => None,
        })
    }
}

fn theme(_state: &State) -> Theme {
    Theme::TokyoNight
}

fn main() -> anyhow::Result<()> {
    simple_logging::log_to_file(
        format!("{}.log", env!("CARGO_PKG_NAME")),
        log::LevelFilter::Warn,
    )?;

    let mut window = window::Settings::default();
    match window::icon::from_file_data(
        include_bytes!("../../resources/images/veelog.ico"),
        Some(image::ImageFormat::Ico),
    ) {
        Ok(v) => window.icon = Some(v),
        Err(_) => error!("Could not load window icon!"),
    }

    Ok(iced::application(State::title, State::update, State::view)
        .subscription(State::rig_update_timer)
        .subscription(State::keyboard_listener)
        .theme(theme)
        .window(window)
        .centered()
        .run()?)
}
