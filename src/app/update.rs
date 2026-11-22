use iced::keyboard::{self, Key, key::Named};
use std::time::Duration;

use iced::{Subscription, Task, keyboard::Modifiers};

use crate::domain::date::shift_month;
use crate::domain::validation::{find_conflicts, validate_teacher_draft, validation_message};

use super::App;
use super::message::{Message, SlotFormMessage, SubjectFormMessage, TeacherFormMessage};
use super::state::{CalendarView, Modal, Screen};

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        let events = iced::event::listen().map(Message::Event);
        let screenshot_due = self.screenshot_path.is_some() && !self.screenshot_requested;

        if screenshot_due {
            Subscription::batch([
                events,
                iced::time::every(Duration::from_millis(4000)).map(|_| Message::ScreenshotTick),
            ])
        } else {
            events
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Previous => {
                self.anchor = match self.view {
                    CalendarView::Week => self.anchor - chrono::Duration::days(7),
                    CalendarView::Month => shift_month(self.anchor, -1),
                };
            }
            Message::Next => {
                self.anchor = match self.view {
                    CalendarView::Week => self.anchor + chrono::Duration::days(7),
                    CalendarView::Month => shift_month(self.anchor, 1),
                };
            }
            Message::Today => {
                self.anchor = chrono::Local::now().date_naive();
            }
            Message::SetView(view) => {
                self.view = view;
            }
            Message::ShowCalendar => {
                self.screen = Screen::Calendar;
                self.modal = None;
            }
            Message::ShowTeachers => {
                self.screen = Screen::Teachers;
                self.modal = None;
            }
            Message::ShowSubjects => {
                self.screen = Screen::Subjects;
                self.modal = None;
            }
            Message::FilterSelected(filter) => {
                self.filter = filter.id;
            }
            Message::OpenNewSlot => {
                self.open_new_slot(None);
            }
            Message::OpenSlot(id) => {
                self.open_slot(id);
            }
            Message::WeekSlotClicked(id) => {
                self.open_slot(id);
            }
            Message::WeekEmptyClicked { weekday, minute } => {
                self.open_new_slot(Some((weekday, minute)));
            }
            Message::SlotMoved {
                id,
                weekday,
                start_minute,
            } => {
                self.move_slot(id, weekday, start_minute);
            }
            Message::MonthDaySelected(date) => {
                self.anchor = date;
                self.view = CalendarView::Week;
            }
            Message::OpenNewTeacher => {
                self.open_new_teacher();
            }
            Message::EditTeacher(id) => {
                self.open_teacher(id);
            }
            Message::AskDeleteTeacher(id) => {
                self.ask_delete_teacher(id);
            }
            Message::DeleteTeacher(id) => {
                self.delete_teacher(id);
            }
            Message::OpenNewSubject => {
                self.open_new_subject();
            }
            Message::EditSubject(id) => {
                self.open_subject(id);
            }
            Message::AskDeleteSubject(id) => {
                self.ask_delete_subject(id);
            }
            Message::DeleteSubject(id) => {
                self.delete_subject(id);
            }
            Message::CloseModal => {
                self.modal = None;
            }
            Message::DismissBanner => {
                self.banner = None;
            }
            Message::TeacherSearch(value) => {
                self.teacher_search = value;
            }
            Message::SubjectSearch(value) => {
                self.subject_search = value;
            }
            Message::SlotForm(form_message) => {
                self.update_slot_form(form_message);
            }
            Message::TeacherForm(form_message) => {
                self.update_teacher_form(form_message);
            }
            Message::SubjectForm(form_message) => {
                self.update_subject_form(form_message);
            }
            Message::RetryStartup => {
                self.retry_boot();
            }
            Message::Quit => {
                return iced::exit();
            }
            Message::ScreenshotTick => {
                if !self.screenshot_requested
                    && let Some(path) = self.screenshot_path.clone()
                {
                    self.screenshot_requested = true;
                    return iced::window::oldest()
                        .and_then(iced::window::screenshot)
                        .map(move |screenshot| {
                            Message::ScreenshotSaved(
                                save_screenshot(&path, &screenshot)
                                    .map_err(|error| error.to_string()),
                            )
                        });
                }
            }
            Message::ScreenshotSaved(result) => {
                self.screenshot_path = None;

                match result {
                    Ok(()) => println!("Скриншот сохранён"),
                    Err(error) => eprintln!("Не удалось сохранить скриншот: {error}"),
                }

                return iced::exit();
            }
            Message::Event(event) => {
                return self.handle_event(event);
            }
        }

        Task::none()
    }

    fn handle_event(&mut self, event: iced::Event) -> Task<Message> {
        let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
        else {
            return Task::none();
        };

        match key {
            Key::Named(Named::Escape) => {
                if self.modal.is_some() {
                    self.modal = None;
                } else if self.banner.is_some() {
                    self.banner = None;
                }
            }
            Key::Character(character)
                if is_command(&modifiers)
                    && character.eq_ignore_ascii_case("n")
                    && self.startup_error.is_none()
                    && self.modal.is_none() =>
            {
                self.screen = Screen::Calendar;
                self.open_new_slot(None);
            }
            Key::Character(character)
                if is_command(&modifiers) && character.eq_ignore_ascii_case("q") =>
            {
                return iced::exit();
            }
            _ => {}
        }

        Task::none()
    }

    fn update_slot_form(&mut self, message: SlotFormMessage) {
        let Some(Modal::Slot(editor)) = &mut self.modal else {
            return;
        };

        match message {
            SlotFormMessage::Kind(kind) => {
                editor.kind = kind;

                if editor.id.is_none()
                    && editor.title.trim().is_empty()
                    && !kind.default_title().is_empty()
                {
                    editor.title = kind.default_title().to_string();
                }

                editor.error = None;
            }
            SlotFormMessage::Weekday(weekday) => {
                editor.weekday = weekday;
                editor.error = None;
            }
            SlotFormMessage::Start(value) => {
                editor.start = value;
                editor.error = None;
            }
            SlotFormMessage::End(value) => {
                editor.end = value;
                editor.error = None;
            }
            SlotFormMessage::Title(value) => {
                editor.title = value;
                editor.error = None;
            }
            SlotFormMessage::Description(value) => {
                editor.description = value;
                editor.error = None;
            }
            SlotFormMessage::ToggleTeacher(teacher_id) => {
                if !editor.selected_teachers.remove(&teacher_id) {
                    editor.selected_teachers.insert(teacher_id);
                }
                editor.error = None;
            }
            SlotFormMessage::Save => {
                self.save_slot();
            }
            SlotFormMessage::Cancel => {
                self.modal = None;
            }
            SlotFormMessage::RequestDelete => {
                editor.confirm_delete = true;
            }
            SlotFormMessage::ConfirmDelete => {
                self.delete_current_slot();
            }
            SlotFormMessage::CancelDelete => {
                editor.confirm_delete = false;
            }
        }
    }

    fn save_slot(&mut self) {
        let Some(Modal::Slot(mut editor)) = self.modal.take() else {
            return;
        };

        let draft = App::slot_draft(&editor);
        let candidate = match draft.validate(&self.teachers) {
            Ok(candidate) => candidate,
            Err(errors) => {
                editor.error = Some(validation_message(&errors));
                self.modal = Some(Modal::Slot(editor));
                return;
            }
        };

        let conflicts = find_conflicts(&candidate, &self.slots, &self.teachers);

        if !conflicts.is_empty() {
            let mut message = String::from("Нельзя сохранить слот:\n");
            message.push_str(
                &conflicts
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            editor.error = Some(message);
            self.modal = Some(Modal::Slot(editor));
            return;
        }

        let Some(database) = self.db.as_mut() else {
            editor.error = Some("База данных недоступна".to_string());
            self.modal = Some(Modal::Slot(editor));
            return;
        };

        match database.save_slot(&candidate) {
            Ok(_) => {
                self.modal = None;

                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => {
                editor.error = Some(error.to_string());
                self.modal = Some(Modal::Slot(editor));
            }
        }
    }

    fn move_slot(&mut self, id: i64, weekday: crate::domain::model::Weekday, start_minute: u16) {
        let Some(slot) = self.slots.iter().find(|slot| slot.id == id) else {
            self.set_banner("Слот больше не существует");
            return;
        };

        let duration = slot.duration_minutes();
        let latest = 1439_u16.saturating_sub(duration);
        let start_minute = start_minute.min(latest);

        if slot.weekday == weekday && slot.start_minute == start_minute {
            return;
        }

        let draft = crate::domain::validation::SlotDraft {
            id: Some(slot.id),
            weekday,
            kind: slot.kind,
            start: crate::domain::format_time(start_minute),
            end: crate::domain::format_time(start_minute + duration),
            title: slot.title.clone(),
            description: slot.description.clone(),
            teacher_ids: slot.teacher_ids.clone(),
        };

        let candidate = match draft.validate(&self.teachers) {
            Ok(candidate) => candidate,
            Err(errors) => {
                self.set_banner(format!(
                    "Не удалось переместить слот: {}",
                    validation_message(&errors)
                ));
                return;
            }
        };

        let conflicts = find_conflicts(&candidate, &self.slots, &self.teachers);

        if !conflicts.is_empty() {
            let mut message = String::from("Не удалось переместить слот:\n");
            message.push_str(
                &conflicts
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            self.set_banner(message);
            return;
        }

        let Some(database) = self.db.as_mut() else {
            self.set_banner("База данных недоступна");
            return;
        };

        match database.save_slot(&candidate) {
            Ok(_) => {
                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => self.set_banner(error.to_string()),
        }
    }

    fn delete_current_slot(&mut self) {
        let id = match &self.modal {
            Some(Modal::Slot(editor)) => editor.id,
            _ => return,
        };

        let Some(id) = id else {
            self.modal = None;
            return;
        };

        let Some(database) = self.db.as_mut() else {
            self.set_banner("База данных недоступна");
            return;
        };

        match database.delete_slot(id) {
            Ok(()) => {
                self.modal = None;

                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => {
                if let Some(Modal::Slot(editor)) = &mut self.modal {
                    editor.error = Some(error.to_string());
                    editor.confirm_delete = false;
                }
            }
        }
    }

    fn update_teacher_form(&mut self, message: TeacherFormMessage) {
        let Some(Modal::Teacher(editor)) = &mut self.modal else {
            return;
        };

        match message {
            TeacherFormMessage::Name(value) => {
                editor.name = value;
                editor.error = None;
            }
            TeacherFormMessage::ToggleSubject(subject_id) => {
                if !editor.selected_subjects.remove(&subject_id) {
                    editor.selected_subjects.insert(subject_id);
                }
                editor.error = None;
            }
            TeacherFormMessage::Note(value) => {
                editor.note = value;
                editor.error = None;
            }
            TeacherFormMessage::Color(color) => {
                editor.color = color;
                editor.error = None;
            }
            TeacherFormMessage::Save => {
                self.save_teacher();
            }
            TeacherFormMessage::Cancel => {
                self.modal = None;
            }
            TeacherFormMessage::RequestDelete => {
                editor.confirm_delete = true;
            }
            TeacherFormMessage::ConfirmDelete => {
                self.delete_current_teacher();
            }
            TeacherFormMessage::CancelDelete => {
                editor.confirm_delete = false;
            }
        }
    }

    fn save_teacher(&mut self) {
        let Some(Modal::Teacher(mut editor)) = self.modal.take() else {
            return;
        };

        let draft = App::teacher_draft(&editor);
        let validated = match validate_teacher_draft(&draft, &self.subjects) {
            Ok(teacher) => teacher,
            Err(errors) => {
                editor.error = Some(validation_message(&errors));
                self.modal = Some(Modal::Teacher(editor));
                return;
            }
        };

        let Some(database) = self.db.as_mut() else {
            editor.error = Some("База данных недоступна".to_string());
            self.modal = Some(Modal::Teacher(editor));
            return;
        };

        match database.save_teacher(&validated) {
            Ok(_) => {
                self.modal = None;

                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => {
                editor.error = Some(error.to_string());
                self.modal = Some(Modal::Teacher(editor));
            }
        }
    }

    fn delete_current_teacher(&mut self) {
        let id = match &self.modal {
            Some(Modal::Teacher(editor)) => editor.id,
            _ => return,
        };

        let Some(id) = id else {
            self.modal = None;
            return;
        };

        self.delete_teacher(id);
    }

    fn update_subject_form(&mut self, message: SubjectFormMessage) {
        let Some(Modal::Subject(editor)) = &mut self.modal else {
            return;
        };

        match message {
            SubjectFormMessage::Name(value) => {
                editor.name = value;
                editor.error = None;
            }
            SubjectFormMessage::Save => {
                self.save_subject();
            }
            SubjectFormMessage::Cancel => {
                self.modal = None;
            }
            SubjectFormMessage::RequestDelete => {
                editor.confirm_delete = true;
            }
            SubjectFormMessage::ConfirmDelete => {
                self.delete_current_subject();
            }
            SubjectFormMessage::CancelDelete => {
                editor.confirm_delete = false;
            }
        }
    }

    fn save_subject(&mut self) {
        let Some(Modal::Subject(mut editor)) = self.modal.take() else {
            return;
        };

        let validated = match App::subject_draft(&editor) {
            Ok(subject) => subject,
            Err(errors) => {
                editor.error = Some(validation_message(&errors));
                self.modal = Some(Modal::Subject(editor));
                return;
            }
        };

        if self.subject_name_exists(&validated.name, validated.id) {
            editor.error = Some("Предмет с таким названием уже существует".to_string());
            self.modal = Some(Modal::Subject(editor));
            return;
        }

        let Some(database) = self.db.as_mut() else {
            editor.error = Some("База данных недоступна".to_string());
            self.modal = Some(Modal::Subject(editor));
            return;
        };

        match database.save_subject(&validated) {
            Ok(_) => {
                self.modal = None;

                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => {
                editor.error = Some(error.to_string());
                self.modal = Some(Modal::Subject(editor));
            }
        }
    }

    fn delete_current_subject(&mut self) {
        let id = match &self.modal {
            Some(Modal::Subject(editor)) => editor.id,
            _ => return,
        };

        let Some(id) = id else {
            self.modal = None;
            return;
        };

        self.delete_subject(id);
    }

    fn delete_subject(&mut self, id: i64) {
        let Some(database) = self.db.as_mut() else {
            self.set_banner("База данных недоступна");
            return;
        };

        match database.delete_subject(id) {
            Ok(()) => {
                self.modal = None;

                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => {
                self.set_banner(error.to_string());
                self.modal = None;
            }
        }
    }

    fn delete_teacher(&mut self, id: i64) {
        let Some(database) = self.db.as_mut() else {
            self.set_banner("База данных недоступна");
            return;
        };

        match database.delete_teacher(id) {
            Ok(()) => {
                self.modal = None;

                if let Err(error) = self.reload() {
                    self.set_banner(error);
                }
            }
            Err(error) => {
                self.set_banner(error.to_string());
                self.modal = None;
            }
        }
    }
}

fn is_command(modifiers: &Modifiers) -> bool {
    modifiers.command() || modifiers.control()
}

fn save_screenshot(
    path: &std::path::Path,
    screenshot: &iced::window::Screenshot,
) -> Result<(), String> {
    let image = image::RgbaImage::from_raw(
        screenshot.size.width,
        screenshot.size.height,
        screenshot.rgba.to_vec(),
    )
    .ok_or_else(|| "некорректный размер изображения".to_string())?;

    image.save(path).map_err(|error| error.to_string())
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::app::state::{CalendarView, Modal};
    use crate::db::Database;
    use crate::domain::model::Weekday;
    use crate::domain::validation::{ValidatedSubject, ValidatedTeacher};

    fn app() -> App {
        App::with_database(
            Database::open_in_memory().unwrap(),
            std::path::PathBuf::from(":memory:"),
        )
    }

    fn add_subject(app: &mut App, name: &str) -> i64 {
        let subject = ValidatedSubject {
            id: None,
            name: name.to_string(),
        };

        app.db.as_mut().unwrap().save_subject(&subject).unwrap()
    }

    fn add_teacher(app: &mut App, name: &str) -> i64 {
        let teacher = ValidatedTeacher {
            id: None,
            name: name.to_string(),
            subject_ids: Vec::new(),
            note: String::new(),
            color: "#2563EB".to_string(),
        };

        app.db.as_mut().unwrap().save_teacher(&teacher).unwrap()
    }

    #[test]
    fn moves_slot_to_new_day_and_time() {
        let mut app = app();
        let teacher_id = add_teacher(&mut app, "Иванов");
        app.reload().unwrap();

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Start("09:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::End("10:30".into())));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));
        let id = app.slots[0].id;

        app.update(Message::SlotMoved {
            id,
            weekday: Weekday::Tuesday,
            start_minute: 600,
        });

        assert_eq!(app.slots[0].weekday, Weekday::Tuesday);
        assert_eq!(app.slots[0].start_minute, 600);
        assert_eq!(app.slots[0].end_minute, 690);
        assert_eq!(app.slots[0].teacher_ids, vec![teacher_id]);
    }

    #[test]
    fn moving_slot_into_conflict_is_rejected() {
        let mut app = app();
        app.anchor = chrono::NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        let teacher_id = add_teacher(&mut app, "Иванов");
        app.reload().unwrap();

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Start("09:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::End("10:30".into())));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Start("11:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::End("12:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));

        assert_eq!(app.slots.len(), 2);
        assert!(
            app.slots
                .iter()
                .all(|slot| slot.teacher_ids == vec![teacher_id])
        );

        let first = app
            .slots
            .iter()
            .find(|slot| slot.start_minute == 540)
            .unwrap()
            .id;
        app.update(Message::SlotMoved {
            id: first,
            weekday: Weekday::Monday,
            start_minute: 660,
        });

        let first_slot = app.slots.iter().find(|slot| slot.id == first).unwrap();
        assert_eq!(first_slot.start_minute, 540);
        assert!(
            app.banner
                .as_ref()
                .is_some_and(|banner| banner.text.contains("Не удалось переместить"))
        );
    }

    #[test]
    fn navigates_week_and_month() {
        let mut app = app();
        app.anchor = chrono::NaiveDate::from_ymd_opt(2026, 11, 18).unwrap();

        app.update(Message::Next);
        assert_eq!(
            app.anchor,
            chrono::NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()
        );

        app.update(Message::SetView(CalendarView::Month));
        app.update(Message::Next);
        assert_eq!(
            app.anchor,
            chrono::NaiveDate::from_ymd_opt(2026, 12, 25).unwrap()
        );

        app.update(Message::Previous);
        assert_eq!(
            app.anchor,
            chrono::NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()
        );
    }

    #[test]
    fn creates_slot_through_form() {
        let mut app = app();
        let teacher_id = add_teacher(&mut app, "Иванов");
        app.reload().unwrap();

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Weekday(
            Weekday::Tuesday,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Start("10:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::End("11:30".into())));
        app.update(Message::SlotForm(SlotFormMessage::Title("Алгебра".into())));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));

        assert!(app.modal.is_none());
        assert_eq!(app.slots.len(), 1);
        assert_eq!(app.slots[0].weekday, Weekday::Tuesday);
        assert_eq!(app.slots[0].start_minute, 600);
        assert_eq!(app.slots[0].teacher_ids, vec![teacher_id]);
    }

    #[test]
    fn blocks_conflicting_slot() {
        let mut app = app();
        let teacher_id = add_teacher(&mut app, "Иванов");
        app.reload().unwrap();

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Start("09:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::End("10:30".into())));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Start("10:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::End("11:00".into())));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));

        assert_eq!(app.slots.len(), 1);
        assert!(matches!(app.modal, Some(Modal::Slot(_))));
        let Some(Modal::Slot(editor)) = &app.modal else {
            unreachable!()
        };
        assert!(editor.error.as_deref().unwrap().contains("пересечение"));
    }

    #[test]
    fn deletes_slot_through_editor() {
        let mut app = app();
        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Save));
        assert_eq!(app.slots.len(), 1);

        let id = app.slots[0].id;
        app.update(Message::OpenSlot(id));
        app.update(Message::SlotForm(SlotFormMessage::RequestDelete));
        app.update(Message::SlotForm(SlotFormMessage::ConfirmDelete));

        assert!(app.slots.is_empty());
        assert!(app.modal.is_none());
    }

    #[test]
    fn deleting_teacher_unassigns_slots_and_resets_filter() {
        let mut app = app();
        let teacher_id = add_teacher(&mut app, "Иванов");
        app.reload().unwrap();
        app.filter = Some(teacher_id);

        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));

        app.update(Message::DeleteTeacher(teacher_id));

        assert!(app.teachers.is_empty());
        assert_eq!(app.slots.len(), 1);
        assert!(app.slots[0].teacher_ids.is_empty());
        assert_eq!(app.filter, None);
    }

    #[test]
    fn month_day_opens_week() {
        let mut app = app();
        app.view = CalendarView::Month;
        let date = chrono::NaiveDate::from_ymd_opt(2026, 12, 3).unwrap();

        app.update(Message::MonthDaySelected(date));

        assert_eq!(app.view, CalendarView::Week);
        assert_eq!(app.anchor, date);
    }

    #[test]
    fn editing_slot_loads_description() {
        let mut app = app();
        let teacher_id = add_teacher(&mut app, "Иванов");
        app.reload().unwrap();
        app.update(Message::OpenNewSlot);
        app.update(Message::SlotForm(SlotFormMessage::Title(
            "Математика".into(),
        )));
        app.update(Message::SlotForm(SlotFormMessage::Description(
            "Описание занятия".into(),
        )));
        app.update(Message::SlotForm(SlotFormMessage::ToggleTeacher(
            teacher_id,
        )));
        app.update(Message::SlotForm(SlotFormMessage::Save));
        let id = app.slots[0].id;

        app.update(Message::OpenSlot(id));
        let Some(Modal::Slot(editor)) = &app.modal else {
            panic!("modal expected");
        };
        assert_eq!(editor.description, "Описание занятия");
    }

    #[test]
    fn teacher_form_saves_and_edits() {
        let mut app = app();
        app.update(Message::OpenNewTeacher);
        app.update(Message::TeacherForm(TeacherFormMessage::Name(
            "Петров".into(),
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::Save));

        assert_eq!(app.teachers.len(), 1);
        assert_eq!(app.teachers[0].name, "Петров");

        let id = app.teachers[0].id;
        app.update(Message::EditTeacher(id));
        app.update(Message::TeacherForm(TeacherFormMessage::Name(
            "Петров Пётр".into(),
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::Save));

        assert_eq!(app.teachers[0].name, "Петров Пётр");
    }

    #[test]
    fn subject_crud_through_forms() {
        let mut app = app();
        app.update(Message::OpenNewSubject);
        app.update(Message::SubjectForm(crate::app::SubjectFormMessage::Name(
            "Математика".into(),
        )));
        app.update(Message::SubjectForm(crate::app::SubjectFormMessage::Save));
        assert_eq!(app.subjects.len(), 1);
        assert_eq!(app.subjects[0].name, "Математика");

        let id = app.subjects[0].id;
        app.update(Message::EditSubject(id));
        app.update(Message::SubjectForm(crate::app::SubjectFormMessage::Name(
            "Алгебра".into(),
        )));
        app.update(Message::SubjectForm(crate::app::SubjectFormMessage::Save));
        assert_eq!(app.subjects[0].name, "Алгебра");

        app.update(Message::OpenNewSubject);
        app.update(Message::SubjectForm(crate::app::SubjectFormMessage::Name(
            "алгебра".into(),
        )));
        app.update(Message::SubjectForm(crate::app::SubjectFormMessage::Save));
        assert_eq!(app.subjects.len(), 1);
        assert!(matches!(app.modal, Some(Modal::Subject(_))));

        let id = app.subjects[0].id;
        app.update(Message::DeleteSubject(id));
        assert!(app.subjects.is_empty());
    }

    #[test]
    fn teacher_form_assigns_multiple_subjects() {
        let mut app = app();
        let math = add_subject(&mut app, "Математика");
        let physics = add_subject(&mut app, "Физика");
        app.reload().unwrap();

        app.update(Message::OpenNewTeacher);
        app.update(Message::TeacherForm(TeacherFormMessage::Name(
            "Иванов".into(),
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::ToggleSubject(
            math,
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::ToggleSubject(
            physics,
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::Save));

        assert_eq!(app.teachers.len(), 1);
        assert_eq!(app.teachers[0].subject_ids, vec![math, physics]);
        assert_eq!(
            app.teacher_subject_names(&app.teachers[0]),
            vec!["Математика", "Физика"]
        );
    }

    #[test]
    fn teacher_form_uses_selected_color() {
        let mut app = app();
        app.update(Message::OpenNewTeacher);
        app.update(Message::TeacherForm(TeacherFormMessage::Color(
            "#DC2626".into(),
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::Name(
            "Сидоров".into(),
        )));
        app.update(Message::TeacherForm(TeacherFormMessage::Save));

        assert_eq!(app.teachers[0].color, "#DC2626");
    }
}
