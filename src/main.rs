use std::path::PathBuf;

use edu_scheduler::app::App;
use edu_scheduler::ui;
use iced::font::{Family, Weight};
use iced::window;
use iced::{Font, Size};

const APP_NAME: &str = "Edu Scheduler";
const APP_TITLE: &str = "Расписание преподавателей";

fn main() -> iced::Result {
    let Options {
        db_path,
        screenshot_path,
        exit,
    } = match Options::parse() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    if exit {
        return Ok(());
    }

    let boot_path = db_path.clone();
    let boot_screenshot = screenshot_path.clone();

    iced::application(
        move || App::boot_with_options(boot_path.clone(), boot_screenshot.clone()),
        App::update,
        ui::view,
    )
    .title(APP_TITLE)
    .theme(iced::Theme::Light)
    .subscription(App::subscription)
    .font(include_bytes!("../assets/fonts/NotoSans-Regular.ttf").as_slice())
    .font(include_bytes!("../assets/fonts/NotoSans-SemiBold.ttf").as_slice())
    .default_font(Font {
        family: Family::Name("Noto Sans"),
        weight: Weight::Normal,
        ..Font::DEFAULT
    })
    .window(window::Settings {
        size: Size::new(1280.0, 840.0),
        min_size: Some(Size::new(1024.0, 720.0)),
        icon: load_icon(),
        ..window::Settings::default()
    })
    .run()
}

struct Options {
    db_path: PathBuf,
    screenshot_path: Option<PathBuf>,
    exit: bool,
}

impl Options {
    fn parse() -> Result<Self, String> {
        let mut db_path = std::env::var_os("EDU_SCHEDULER_DB")
            .map(PathBuf::from)
            .unwrap_or_else(default_db_path);
        let mut screenshot_path = std::env::var_os("EDU_SCREENSHOT").map(PathBuf::from);
        let mut arguments = std::env::args().skip(1);
        let mut exit = false;

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--help" | "-h" => {
                    print_help();
                    exit = true;
                    break;
                }
                "--version" | "-V" => {
                    println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
                    exit = true;
                    break;
                }
                "--db" => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "После --db нужен путь к файлу".to_string())?;
                    db_path = PathBuf::from(value);
                }
                other if other.starts_with("--db=") => {
                    db_path = PathBuf::from(&other["--db=".len()..]);
                }
                "--screenshot" => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "После --screenshot нужен путь к PNG".to_string())?;
                    screenshot_path = Some(PathBuf::from(value));
                }
                other if other.starts_with("--screenshot=") => {
                    screenshot_path = Some(PathBuf::from(&other["--screenshot=".len()..]));
                }
                other => {
                    return Err(format!(
                        "Неизвестный аргумент: {other}. Используйте --help."
                    ));
                }
            }
        }

        Ok(Self {
            db_path,
            screenshot_path,
            exit,
        })
    }
}

fn default_db_path() -> PathBuf {
    directories::ProjectDirs::from("ru", "v-aleks", "edu-scheduler")
        .map(|directories| directories.data_dir().join("schedule.db"))
        .unwrap_or_else(|| PathBuf::from("schedule.db"))
}

fn load_icon() -> Option<window::Icon> {
    let bytes = include_bytes!("../assets/icons/icon-128.png");
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();

    window::icon::from_rgba(image.into_raw(), width, height).ok()
}

fn print_help() {
    println!(
        "{APP_NAME} — офлайн-расписание преподавателей\n\
         \n\
         Использование: edu-scheduler [--db <путь>] [--screenshot <путь>]\n\
         \n\
         --db <путь>          путь к файлу SQLite (по умолчанию в каталоге данных)\n\
         --screenshot <путь>  сохранить скриншот окна в PNG и выйти\n\
         --help, -h           показать эту справку\n\
         --version, -V        показать версию\n\
         \n\
         Переменная окружения EDU_SCHEDULER_DB задаёт путь к БД по умолчанию."
    );
}
