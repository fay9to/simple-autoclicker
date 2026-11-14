//! Translations. English, Russian, Spanish and German are built in; the
//! language is picked from the Windows UI language on the first run and can be
//! changed from the globe button in the title bar.

use windows::Win32::Globalization::GetUserDefaultUILanguage;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Ru,
    Es,
    De,
}

impl Lang {
    pub const ALL: [Lang; 4] = [Lang::En, Lang::Ru, Lang::Es, Lang::De];

    /// Value stored in the settings file.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Es => "es",
            Lang::De => "de",
        }
    }

    /// Name written in the language itself, so the menu is always readable.
    pub fn native(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Ru => "Русский",
            Lang::Es => "Español",
            Lang::De => "Deutsch",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Some(Lang::En),
            "ru" | "russian" => Some(Lang::Ru),
            "es" | "spanish" => Some(Lang::Es),
            "de" | "german" => Some(Lang::De),
            _ => None,
        }
    }

    /// Picks a language from the Windows UI language on the very first run.
    pub fn detect() -> Self {
        let id = unsafe { GetUserDefaultUILanguage() } & 0x3FF;
        match id {
            0x19 => Lang::Ru,
            0x0A => Lang::Es,
            0x07 => Lang::De,
            _ => Lang::En,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Ok,
    Cancel,
    Back,
    More,
    Ready,
    Running,
    Stopped,
    Recording,
    Playing,
    ClickInterval,
    Hours,
    Mins,
    Secs,
    Millis,
    ClickOptions,
    MouseButton,
    Left,
    Right,
    Middle,
    ClickType,
    Single,
    Double,
    ClickRepeat,
    Repeat,
    Times,
    UntilStopped,
    CursorPosition,
    CurrentLocation,
    PickLocation,
    PickHint,
    HotkeySetting,
    RecordPlayback,
    RecordHint,
    Playback,
    Record,
    PlaybackHint,
    RecordHintTile,
    StopPlayback,
    StopRecording,
    Details,
    Clicks,
    Moves,
    Events,
    ClearRecording,
    NothingRecorded,
    Settings,
    Hotkeys,
    ThemeSystem,
    ThemeLight,
    ThemeDark,
    HowToRecord,
    About,
    HowToText,
    StartStop,
    PlaybackStop,
    RecordStop,
    MinimizePlaying,
    MinimizeRecording,
    DialogHotkey,
    DialogSettings,
    DialogHotkeys,
    DialogPick,
    HookFailed,
    NothingRecordedMessage,
    HotkeysBusy,
    TipTheme,
    TipLanguage,
    TipMinimize,
    TipClose,
    TipPick,
    RecordingDetailsHint,
}

struct Line {
    en: &'static str,
    ru: &'static str,
    es: &'static str,
    de: &'static str,
}

impl Key {
    fn line(self) -> Line {
        use Key::*;
        match self {
            Ok => Line {
                en: "Ok",
                ru: "Ок",
                es: "Aceptar",
                de: "OK",
            },
            Cancel => Line {
                en: "Cancel",
                ru: "Отмена",
                es: "Cancelar",
                de: "Abbrechen",
            },
            Back => Line {
                en: "Back",
                ru: "Назад",
                es: "Atrás",
                de: "Zurück",
            },
            More => Line {
                en: "More",
                ru: "Ещё",
                es: "Más",
                de: "Mehr",
            },
            Ready => Line {
                en: "Ready",
                ru: "Готово",
                es: "Listo",
                de: "Bereit",
            },
            Running => Line {
                en: "Running",
                ru: "Работает",
                es: "En marcha",
                de: "Läuft",
            },
            Stopped => Line {
                en: "Stopped",
                ru: "Остановлено",
                es: "Detenido",
                de: "Gestoppt",
            },
            Recording => Line {
                en: "Recording",
                ru: "Запись",
                es: "Grabando",
                de: "Aufnahme",
            },
            Playing => Line {
                en: "Playing",
                ru: "Воспроизведение",
                es: "Reproduciendo",
                de: "Wiedergabe",
            },
            ClickInterval => Line {
                en: "Click interval",
                ru: "Интервал кликов",
                es: "Intervalo de clic",
                de: "Klickintervall",
            },
            Hours => Line {
                en: "hours",
                ru: "часы",
                es: "horas",
                de: "Stunden",
            },
            Mins => Line {
                en: "mins",
                ru: "минуты",
                es: "minutos",
                de: "Minuten",
            },
            Secs => Line {
                en: "secs",
                ru: "секунды",
                es: "segundos",
                de: "Sekunden",
            },
            Millis => Line {
                en: "milliseconds",
                ru: "миллисекунды",
                es: "milisegundos",
                de: "Millisekunden",
            },
            ClickOptions => Line {
                en: "Click options",
                ru: "Параметры клика",
                es: "Opciones de clic",
                de: "Klickoptionen",
            },
            MouseButton => Line {
                en: "Mouse button",
                ru: "Кнопка мыши",
                es: "Botón del ratón",
                de: "Maustaste",
            },
            Left => Line {
                en: "Left",
                ru: "Левая",
                es: "Izquierdo",
                de: "Links",
            },
            Right => Line {
                en: "Right",
                ru: "Правая",
                es: "Derecho",
                de: "Rechts",
            },
            Middle => Line {
                en: "Middle",
                ru: "Средняя",
                es: "Medio",
                de: "Mitte",
            },
            ClickType => Line {
                en: "Click type",
                ru: "Тип клика",
                es: "Tipo de clic",
                de: "Klickart",
            },
            Single => Line {
                en: "Single",
                ru: "Одинарный",
                es: "Simple",
                de: "Einfach",
            },
            Double => Line {
                en: "Double",
                ru: "Двойной",
                es: "Doble",
                de: "Doppelt",
            },
            ClickRepeat => Line {
                en: "Click repeat",
                ru: "Повтор кликов",
                es: "Repetición",
                de: "Wiederholung",
            },
            Repeat => Line {
                en: "Repeat",
                ru: "Повторить",
                es: "Repetir",
                de: "Wiederholen",
            },
            Times => Line {
                en: "times",
                ru: "раз",
                es: "veces",
                de: "Mal",
            },
            UntilStopped => Line {
                en: "Repeat until stopped",
                ru: "Повторять до остановки",
                es: "Repetir hasta detener",
                de: "Bis zum Stopp wiederholen",
            },
            CursorPosition => Line {
                en: "Cursor position",
                ru: "Позиция курсора",
                es: "Posición del cursor",
                de: "Cursorposition",
            },
            CurrentLocation => Line {
                en: "Current location",
                ru: "Текущее положение",
                es: "Posición actual",
                de: "Aktuelle Position",
            },
            PickLocation => Line {
                en: "Pick location",
                ru: "Выбрать точку",
                es: "Elegir punto",
                de: "Position wählen",
            },
            PickHint => Line {
                en: "Point the cursor at the spot you need",
                ru: "Наведите курсор на нужную точку",
                es: "Apunta el cursor al lugar deseado",
                de: "Cursor auf die gewünschte Stelle richten",
            },
            HotkeySetting => Line {
                en: "Hotkey setting",
                ru: "Горячие клавиши",
                es: "Atajo de teclado",
                de: "Tastenkürzel",
            },
            RecordPlayback => Line {
                en: "Record & Playback",
                ru: "Запись и воспроизведение",
                es: "Grabar y reproducir",
                de: "Aufnehmen & Abspielen",
            },
            RecordHint => Line {
                en: "Record clicks and mouse movement, then play them back to repeat the same sequence.",
                ru: "Запишите клики и движение мыши, а затем воспроизведите, чтобы повторить ту же последовательность.",
                es: "Graba clics y movimientos del ratón y reprodúcelos para repetir la misma secuencia.",
                de: "Klicks und Mausbewegungen aufnehmen und abspielen, um die Sequenz zu wiederholen.",
            },
            Playback => Line {
                en: "Playback",
                ru: "Воспроизвести",
                es: "Reproducir",
                de: "Abspielen",
            },
            Record => Line {
                en: "Record",
                ru: "Запись",
                es: "Grabar",
                de: "Aufnehmen",
            },
            PlaybackHint => Line {
                en: "Repeat the recording",
                ru: "Повторить запись",
                es: "Repetir la grabación",
                de: "Aufnahme wiederholen",
            },
            RecordHintTile => Line {
                en: "Capture clicks and movement",
                ru: "Записать клики и движение",
                es: "Grabar clics y movimiento",
                de: "Klicks und Bewegung aufnehmen",
            },
            StopPlayback => Line {
                en: "Stop playback",
                ru: "Остановить воспроизведение",
                es: "Detener reproducción",
                de: "Wiedergabe stoppen",
            },
            StopRecording => Line {
                en: "Stop recording",
                ru: "Остановить запись",
                es: "Detener grabación",
                de: "Aufnahme stoppen",
            },
            Details => Line {
                en: "Recording",
                ru: "Запись",
                es: "Grabación",
                de: "Aufnahme",
            },
            Clicks => Line {
                en: "Clicks",
                ru: "Клики",
                es: "Clics",
                de: "Klicks",
            },
            Moves => Line {
                en: "Moves",
                ru: "Движения",
                es: "Movimientos",
                de: "Bewegungen",
            },
            Events => Line {
                en: "Events",
                ru: "События",
                es: "Eventos",
                de: "Ereignisse",
            },
            ClearRecording => Line {
                en: "Clear recording",
                ru: "Очистить запись",
                es: "Borrar grabación",
                de: "Aufnahme löschen",
            },
            NothingRecorded => Line {
                en: "Nothing recorded yet",
                ru: "Пока ничего не записано",
                es: "Aún no hay nada grabado",
                de: "Noch nichts aufgenommen",
            },
            Settings => Line {
                en: "Settings…",
                ru: "Настройки…",
                es: "Ajustes…",
                de: "Einstellungen…",
            },
            Hotkeys => Line {
                en: "Hotkeys…",
                ru: "Горячие клавиши…",
                es: "Atajos…",
                de: "Tastenkürzel…",
            },
            ThemeSystem => Line {
                en: "System",
                ru: "Системная",
                es: "Sistema",
                de: "System",
            },
            ThemeLight => Line {
                en: "Light",
                ru: "Светлая",
                es: "Claro",
                de: "Hell",
            },
            ThemeDark => Line {
                en: "Dark",
                ru: "Тёмная",
                es: "Oscuro",
                de: "Dunkel",
            },
            HowToRecord => Line {
                en: "How to record and playback?",
                ru: "Как записывать и воспроизводить?",
                es: "¿Cómo grabar y reproducir?",
                de: "Aufnehmen und Abspielen?",
            },
            About => Line {
                en: "About",
                ru: "О программе",
                es: "Acerca de",
                de: "Über",
            },
            HowToText => Line {
                en: "Press Record, perform the clicks and mouse movements you need, then press Stop.\n\nPlayback repeats the recording at the same speed. Attach hotkeys to control it from any app.",
                ru: "Нажмите «Запись», выполните нужные клики и движения мыши, затем нажмите «Стоп».\n\nВоспроизведение повторяет запись с той же скоростью. Назначьте горячие клавиши, чтобы управлять записью из любого приложения.",
                es: "Pulsa Grabar, realiza los clics y movimientos de ratón que necesites y pulsa Detener.\n\nLa reproducción repite la grabación a la misma velocidad. Asigna atajos para controlarla desde cualquier aplicación.",
                de: "Aufnehmen drücken, die gewünschten Klicks und Mausbewegungen ausführen und dann Stopp drücken.\n\nDie Wiedergabe wiederholt die Aufnahme mit derselben Geschwindigkeit. Tastenkürzel ermöglichen die Steuerung aus jeder Anwendung.",
            },
            StartStop => Line {
                en: "Start / Stop",
                ru: "Старт / Стоп",
                es: "Iniciar / Detener",
                de: "Start / Stopp",
            },
            PlaybackStop => Line {
                en: "Playback / Stop",
                ru: "Воспроизведение / Стоп",
                es: "Reproducir / Detener",
                de: "Abspielen / Stopp",
            },
            RecordStop => Line {
                en: "Record / Stop",
                ru: "Запись / Стоп",
                es: "Grabar / Detener",
                de: "Aufnehmen / Stopp",
            },
            MinimizePlaying => Line {
                en: "Minimize when playing",
                ru: "Сворачивать при воспроизведении",
                es: "Minimizar al reproducir",
                de: "Beim Abspielen minimieren",
            },
            MinimizeRecording => Line {
                en: "Minimize when recording",
                ru: "Сворачивать при записи",
                es: "Minimizar al grabar",
                de: "Beim Aufnehmen minimieren",
            },
            DialogHotkey => Line {
                en: "Hotkey",
                ru: "Горячая клавиша",
                es: "Atajo de teclado",
                de: "Tastenkürzel",
            },
            DialogSettings => Line {
                en: "Settings",
                ru: "Настройки",
                es: "Ajustes",
                de: "Einstellungen",
            },
            DialogHotkeys => Line {
                en: "Hotkeys",
                ru: "Горячие клавиши",
                es: "Atajos",
                de: "Tastenkürzel",
            },
            DialogPick => Line {
                en: "Pick location",
                ru: "Выбор точки",
                es: "Elegir punto",
                de: "Position wählen",
            },
            HookFailed => Line {
                en: "Could not start recording: the system did not allow the mouse hook.",
                ru: "Не удалось начать запись: система не разрешила перехват мыши.",
                es: "No se pudo iniciar la grabación: el sistema no permitió el enlace del ratón.",
                de: "Aufnahme konnte nicht gestartet werden: Das System hat den Maus-Hook nicht erlaubt.",
            },
            NothingRecordedMessage => Line {
                en: "Record something first, using the Record button.",
                ru: "Сначала что-нибудь запишите кнопкой «Запись».",
                es: "Graba algo primero con el botón Grabar.",
                de: "Nehme zuerst etwas mit der Schaltfläche Aufnehmen auf.",
            },
            HotkeysBusy => Line {
                en: "Those hotkeys are already used by another program. The previous ones were kept.",
                ru: "Эти клавиши уже заняты другой программой. Прежние остались.",
                es: "Otro programa ya usa esos atajos. Se conservaron los anteriores.",
                de: "Diese Tastenkürzel werden bereits von einem anderen Programm verwendet. Die vorherigen bleiben erhalten.",
            },
            TipTheme => Line {
                en: "Appearance: system, light or dark",
                ru: "Оформление: системное, светлое или тёмное",
                es: "Apariencia: sistema, claro u oscuro",
                de: "Erscheinungsbild: System, hell oder dunkel",
            },
            TipLanguage => Line {
                en: "Language",
                ru: "Язык",
                es: "Idioma",
                de: "Sprache",
            },
            TipMinimize => Line {
                en: "Minimize",
                ru: "Свернуть",
                es: "Minimizar",
                de: "Minimieren",
            },
            TipClose => Line {
                en: "Close",
                ru: "Закрыть",
                es: "Cerrar",
                de: "Schließen",
            },
            TipPick => Line {
                en: "Capture the current cursor position",
                ru: "Запомнить текущую позицию курсора",
                es: "Capturar la posición actual del cursor",
                de: "Aktuelle Cursorposition übernehmen",
            },
            RecordingDetailsHint => Line {
                en: "Playback repeats every click and movement at the recorded speed.",
                ru: "Воспроизведение повторяет каждый клик и движение с записанной скоростью.",
                es: "La reproducción repite cada clic y movimiento a la velocidad grabada.",
                de: "Die Wiedergabe wiederholt jeden Klick und jede Bewegung mit der aufgenommenen Geschwindigkeit.",
            },
        }
    }
}

pub fn t(lang: Lang, key: Key) -> &'static str {
    let line = key.line();
    match lang {
        Lang::En => line.en,
        Lang::Ru => line.ru,
        Lang::Es => line.es,
        Lang::De => line.de,
    }
}

pub fn start_button(lang: Lang, key: &str) -> String {
    match lang {
        Lang::En => format!("Start ({key})"),
        Lang::Ru => format!("Старт ({key})"),
        Lang::Es => format!("Iniciar ({key})"),
        Lang::De => format!("Start ({key})"),
    }
}

pub fn stop_button(lang: Lang, key: &str) -> String {
    match lang {
        Lang::En => format!("Stop ({key})"),
        Lang::Ru => format!("Стоп ({key})"),
        Lang::Es => format!("Detener ({key})"),
        Lang::De => format!("Stopp ({key})"),
    }
}

pub fn hotkey_busy(lang: Lang, key: &str) -> String {
    match lang {
        Lang::En => format!("Could not assign {key} - another program may already use this key."),
        Lang::Ru => format!("Не удалось назначить {key}: возможно, клавиша занята другой программой."),
        Lang::Es => format!("No se pudo asignar {key}: puede que otro programa ya use esta tecla."),
        Lang::De => format!("{key} konnte nicht zugewiesen werden - möglicherweise wird die Taste bereits verwendet."),
    }
}

pub fn about_text(lang: Lang) -> String {
    let version = env!("CARGO_PKG_VERSION");
    match lang {
        Lang::En => format!(
            "Simple Auto Clicker\nVersion {version}\n\nA fast, lightweight auto clicker with record & playback.\nMIT License"
        ),
        Lang::Ru => format!(
            "Simple Auto Clicker\nВерсия {version}\n\nБыстрый и лёгкий автокликер с записью и воспроизведением.\nЛицензия MIT"
        ),
        Lang::Es => format!(
            "Simple Auto Clicker\nVersión {version}\n\nUn autoclicker rápido y ligero con grabación y reproducción.\nLicencia MIT"
        ),
        Lang::De => format!(
            "Simple Auto Clicker\nVersion {version}\n\nEin schneller, schlanker Auto-Klicker mit Aufnahme und Wiedergabe.\nMIT-Lizenz"
        ),
    }
}
