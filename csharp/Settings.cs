using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Windows.Forms;

namespace SimpleAutoClicker;

/// <summary>
/// Настройки приложения. Хранятся рядом с пользовательским профилем,
/// в %LOCALAPPDATA%\SimpleAutoClicker\settings.ini — удаление файла сбрасывает всё.
/// </summary>
internal sealed class AppSettings
{
    private static readonly string Folder = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "SimpleAutoClicker");

    private static readonly string FilePath = Path.Combine(Folder, "settings.ini");

    public static AppSettings Current { get; private set; } = new();

    public AppTheme Theme { get; set; } = AppTheme.System;

    /// <summary>Виртуальный код клавиши Start/Stop, по умолчанию F6.</summary>
    public int Hotkey { get; set; } = 0x75;

    public int PlaybackHotkey { get; set; }

    public int RecordHotkey { get; set; }

    public bool MinimizeWhenPlaying { get; set; }

    public bool MinimizeWhenRecording { get; set; }

    public static void Load()
    {
        var settings = new AppSettings();
        try
        {
            if (File.Exists(FilePath))
            {
                foreach (string line in File.ReadAllLines(FilePath))
                {
                    int split = line.IndexOf('=');
                    if (split <= 0)
                    {
                        continue;
                    }

                    string key = line[..split].Trim();
                    string value = line[(split + 1)..].Trim();
                    settings.Apply(key, value);
                }
            }
        }
        catch (IOException)
        {
            // битый или недоступный файл — работаем со значениями по умолчанию
        }
        catch (UnauthorizedAccessException)
        {
            // нет прав на чтение — тоже значения по умолчанию
        }

        Current = settings;
    }

    public void Save()
    {
        try
        {
            Directory.CreateDirectory(Folder);
            File.WriteAllLines(FilePath, new[]
            {
                "theme=" + Theme.ToString().ToLowerInvariant(),
                "hotkey=" + Hotkey.ToString(CultureInfo.InvariantCulture),
                "playback_hotkey=" + PlaybackHotkey.ToString(CultureInfo.InvariantCulture),
                "record_hotkey=" + RecordHotkey.ToString(CultureInfo.InvariantCulture),
                "minimize_when_playing=" + (MinimizeWhenPlaying ? "1" : "0"),
                "minimize_when_recording=" + (MinimizeWhenRecording ? "1" : "0"),
            });
        }
        catch (IOException)
        {
            // настройки — не повод падать
        }
        catch (UnauthorizedAccessException)
        {
            // см. выше
        }
    }

    private void Apply(string key, string value)
    {
        switch (key)
        {
            case "theme":
                if (Enum.TryParse(value, true, out AppTheme theme))
                {
                    Theme = theme;
                }

                break;

            case "hotkey":
                Hotkey = ParseKey(value, Hotkey);
                break;

            case "playback_hotkey":
                PlaybackHotkey = ParseKey(value, PlaybackHotkey);
                break;

            case "record_hotkey":
                RecordHotkey = ParseKey(value, RecordHotkey);
                break;

            case "minimize_when_playing":
                MinimizeWhenPlaying = value == "1";
                break;

            case "minimize_when_recording":
                MinimizeWhenRecording = value == "1";
                break;
        }
    }

    private static int ParseKey(string value, int fallback)
    {
        return int.TryParse(value, NumberStyles.Integer, CultureInfo.InvariantCulture, out int v)
            ? v
            : fallback;
    }
}

internal static class AppControl
{
    /// <summary>
    /// Перезапускает приложение. Тема применяется при создании окон,
    /// поэтому после смены оформления проще стартовать заново.
    /// </summary>
    public static void Restart()
    {
        string? exe = Environment.ProcessPath;
        if (!string.IsNullOrEmpty(exe))
        {
            try
            {
                Process.Start(new ProcessStartInfo(exe)
                {
                    UseShellExecute = true,
                    WorkingDirectory = AppContext.BaseDirectory,
                });
            }
            catch (Exception)
            {
                // если запустить не удалось, просто закрываемся — настройки уже сохранены
            }
        }

        Application.Exit();
    }
}
