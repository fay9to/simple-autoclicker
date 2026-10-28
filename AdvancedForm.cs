using System.Diagnostics;

namespace SimpleAutoClicker;

/// <summary>Окно «Simple Auto Clicker Advanced»: запись и воспроизведение кликов и движений мыши.</summary>
internal sealed class AdvancedForm : Form
{
    private const int HotkeyPlayback = 0x0AD1;
    private const int HotkeyRecord = 0x0AD2;

    private readonly ModernButton _btnPlay = new();
    private readonly ModernButton _btnRecord = new();
    private readonly ModernButton _btnMore = new();
    private readonly Button _btnOptions = new();
    private readonly Button _btnHelp = new();

    private readonly Label _lblHint = new();
    private readonly Label _lblStatus = new();

    private readonly MouseHook _hook = new();
    private readonly List<RecordedEvent> _events = new();
    private readonly System.Windows.Forms.Timer _uiTimer = new();

    private Stopwatch? _recordClock;
    private volatile bool _recording;
    private volatile bool _playing;
    private Thread? _playThread;

    private int _lastMoveMs = -1000;
    private int _lastMoveX;
    private int _lastMoveY;

    private int _playbackHotkey;
    private int _recordHotkey;
    private bool _minimizeWhenPlaying;
    private bool _minimizeWhenRecording;
    private bool _restoreAfterOperation;

    public AdvancedForm()
    {
        Text = "Simple Auto Clicker Advanced";
        ClientSize = new Size(380, 220);
        FormBorderStyle = FormBorderStyle.FixedSingle;
        MaximizeBox = false;
        MinimizeBox = true;
        StartPosition = FormStartPosition.CenterScreen;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F);
        DoubleBuffered = true;

        _playbackHotkey = AppSettings.Current.PlaybackHotkey;
        _recordHotkey = AppSettings.Current.RecordHotkey;
        _minimizeWhenPlaying = AppSettings.Current.MinimizeWhenPlaying;
        _minimizeWhenRecording = AppSettings.Current.MinimizeWhenRecording;

        BuildUi();
        WireEvents();

        _hook.MouseEvent += OnMouseEvent;
        _hook.MouseMoved += OnMouseMoved;

        _uiTimer.Interval = 200;
        _uiTimer.Tick += (_, _) => UpdateState();

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }

    protected override void OnShown(EventArgs e)
    {
        base.OnShown(e);
        RegisterHotkeys();
        _uiTimer.Start();
        UpdateState();
    }

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        StopRecording();
        StopPlayback();
        _uiTimer.Stop();
        _hook.Dispose();
        Native.UnregisterHotKey(Handle, HotkeyPlayback);
        Native.UnregisterHotKey(Handle, HotkeyRecord);
        base.OnFormClosing(e);
    }

    protected override void WndProc(ref Message m)
    {
        if (m.Msg == Native.WM_HOTKEY)
        {
            int id = m.WParam.ToInt32();
            if (id == HotkeyPlayback)
            {
                TogglePlayback();
                return;
            }

            if (id == HotkeyRecord)
            {
                ToggleRecording();
                return;
            }
        }

        base.WndProc(ref m);
    }

    // ------------------------------------------------------------------ UI

    private void BuildUi()
    {
        _btnOptions.Text = "Options";
        _btnOptions.SetBounds(14, 12, 74, 26);
        StyleMenuButton(_btnOptions, ShowOptionsMenu);

        _btnHelp.Text = "Help";
        _btnHelp.SetBounds(94, 12, 50, 26);
        StyleMenuButton(_btnHelp, ShowHelpMenu);

        _lblHint.Text = "Record clicks and mouse movement, then play them back to repeat the same sequence.";
        _lblHint.Bounds = new Rectangle(18, 54, 344, 34);
        _lblHint.Font = Theme.F(9F);
        _lblHint.ForeColor = Theme.Muted;
        _lblHint.BackColor = Theme.Bg;
        _lblHint.TextAlign = ContentAlignment.MiddleLeft;

        _btnPlay.Icon = IconKind.Play;
        _btnPlay.IconSize = 26;
        _btnPlay.IconColor = Theme.Green;
        _btnPlay.Style = BtnStyle.Secondary;
        _btnPlay.SetBounds(24, 100, 104, 76);

        _btnRecord.Icon = IconKind.Record;
        _btnRecord.IconSize = 24;
        _btnRecord.IconColor = Theme.Danger;
        _btnRecord.Style = BtnStyle.Secondary;
        _btnRecord.SetBounds(138, 100, 104, 76);

        _btnMore.Icon = IconKind.More;
        _btnMore.IconSize = 14;
        _btnMore.Text = "More";
        _btnMore.Style = BtnStyle.Secondary;
        _btnMore.SetBounds(252, 100, 104, 76);

        _lblStatus.Text = "Events: 0";
        _lblStatus.Bounds = new Rectangle(24, 186, 332, 20);
        _lblStatus.Font = Theme.F(8.5F);
        _lblStatus.ForeColor = Theme.Muted;
        _lblStatus.BackColor = Theme.Bg;
        _lblStatus.TextAlign = ContentAlignment.MiddleLeft;

        Controls.AddRange(new Control[]
        {
            _btnOptions, _btnHelp, _lblHint,
            _btnPlay, _btnRecord, _btnMore, _lblStatus,
        });
    }

    private void StyleMenuButton(Button button, Action onClick)
    {
        button.FlatStyle = FlatStyle.Flat;
        button.FlatAppearance.BorderSize = 0;
        button.FlatAppearance.MouseOverBackColor = Theme.Bg;
        button.BackColor = Theme.Bg;
        button.ForeColor = Theme.Muted;
        button.Font = Theme.F(9F);
        button.Cursor = Cursors.Hand;
        button.Click += (_, _) => onClick();
        button.MouseEnter += (_, _) => button.ForeColor = Theme.Ink;
        button.MouseLeave += (_, _) => button.ForeColor = Theme.Muted;
    }

    private void WireEvents()
    {
        _btnPlay.Click += (_, _) => TogglePlayback();
        _btnRecord.Click += (_, _) => ToggleRecording();
        _btnMore.Click += (_, _) => ShowMoreMenu();
    }

    // ------------------------------------------------------------- recording

    private void ToggleRecording()
    {
        if (_recording)
        {
            StopRecording();
        }
        else
        {
            StartRecording();
        }
    }

    private void StartRecording()
    {
        if (_playing)
        {
            return;
        }

        _events.Clear();
        _lastMoveMs = -1000;
        _recordClock = Stopwatch.StartNew();
        _recording = true;

        if (!_hook.Start())
        {
            _recording = false;
            _recordClock = null;
            UpdateState();
            Msg.Warn(this, "Could not start recording: the system did not allow the mouse hook.");
            return;
        }

        if (_minimizeWhenRecording)
        {
            _restoreAfterOperation = true;
            WindowState = FormWindowState.Minimized;
        }

        UpdateState();
    }

    private void StopRecording()
    {
        if (!_recording)
        {
            return;
        }

        _recording = false;
        _hook.Stop();
        _recordClock?.Stop();
        RestoreIfNeeded();
        UpdateState();
    }

    private void OnMouseEvent(MouseButtonKind button, bool down, int x, int y)
    {
        if (!_recording || _recordClock is null)
        {
            return;
        }

        // клики по собственному окну не записываем: иначе при воспроизведении
        // они снова нажимали бы Record/Stop и ломали сценарий
        if (WindowState == FormWindowState.Normal && Bounds.Contains(x, y))
        {
            return;
        }

        _events.Add(new RecordedEvent
        {
            Button = button,
            Down = down,
            X = x,
            Y = y,
            TimeMs = _recordClock.ElapsedMilliseconds,
        });
    }

    private void OnMouseMoved(int x, int y)
    {
        if (!_recording || _recordClock is null)
        {
            return;
        }

        // движение пишем не чаще ~60 раз в секунду и только если курсор реально сместился,
        // иначе список раздувается десятками тысяч точек
        long now = _recordClock.ElapsedMilliseconds;
        if (now - _lastMoveMs < 12 && Math.Abs(x - _lastMoveX) < 3 && Math.Abs(y - _lastMoveY) < 3)
        {
            return;
        }

        _lastMoveMs = (int)now;
        _lastMoveX = x;
        _lastMoveY = y;

        _events.Add(new RecordedEvent
        {
            Move = true,
            X = x,
            Y = y,
            TimeMs = now,
        });
    }

    // -------------------------------------------------------------- playback

    private void TogglePlayback()
    {
        if (_playing)
        {
            StopPlayback();
        }
        else
        {
            StartPlayback();
        }
    }

    private void StartPlayback()
    {
        if (_recording)
        {
            return;
        }

        if (_events.Count == 0)
        {
            Msg.Info(this, "Record something first, using the Record button.");
            return;
        }

        _playing = true;
        _restoreAfterOperation = false;
        if (_minimizeWhenPlaying)
        {
            _restoreAfterOperation = true;
            WindowState = FormWindowState.Minimized;
        }

        var events = _events.ToArray();
        _playThread = new Thread(() => PlaybackLoop(events))
        {
            IsBackground = true,
            Name = "auto-clicker-playback",
        };
        _playThread.Start();
        UpdateState();
    }

    private void PlaybackLoop(RecordedEvent[] events)
    {
        try
        {
            var clock = Stopwatch.StartNew();
            foreach (var ev in events)
            {
                while (_playing)
                {
                    long left = ev.TimeMs - clock.ElapsedMilliseconds;
                    if (left <= 0)
                    {
                        break;
                    }

                    Thread.Sleep((int)Math.Min(left, 20));
                }

                if (!_playing)
                {
                    return;
                }

                Native.MoveAbsolute(ev.X, ev.Y);
                if (!ev.Move)
                {
                    Native.SendButton(ev.Button, ev.Down);
                }
            }
        }
        finally
        {
            _playing = false;
            if (IsHandleCreated && !IsDisposed)
            {
                BeginInvoke(new Action(() =>
                {
                    RestoreIfNeeded();
                    UpdateState();
                }));
            }
        }
    }

    private void StopPlayback()
    {
        _playing = false;
        _playThread?.Join(300);
        _playThread = null;
        RestoreIfNeeded();
        UpdateState();
    }

    // --------------------------------------------------------------- helpers

    private void RestoreIfNeeded()
    {
        if (!_restoreAfterOperation)
        {
            return;
        }

        _restoreAfterOperation = false;
        if (IsHandleCreated && !IsDisposed)
        {
            WindowState = FormWindowState.Normal;
            Activate();
        }
    }

    private void UpdateState()
    {
        _btnPlay.Icon = _playing ? IconKind.Stop : IconKind.Play;
        _btnPlay.IconColor = _playing ? Theme.Ink : Theme.Green;
        _btnRecord.Icon = _recording ? IconKind.Stop : IconKind.Record;
        _btnRecord.IconColor = _recording ? Theme.Ink : Theme.Danger;
        _btnRecord.Enabled = !_playing;
        _btnPlay.Enabled = !_recording;
        _btnMore.Enabled = !_playing && !_recording;

        if (_recording)
        {
            _lblStatus.Text = $"Recording… events: {_events.Count}";
        }
        else if (_playing)
        {
            _lblStatus.Text = "Playing back…";
        }
        else
        {
            int moves = _events.Count(e => e.Move);
            int clicks = _events.Count(e => !e.Move);
            _lblStatus.Text = $"Recorded: {clicks / 2} clicks, {moves} move points";
        }
    }

    // ----------------------------------------------------------------- menus

    private void ShowOptionsMenu()
    {
        var menu = Theme.Menu();
        menu.ShowCheckMargin = true;
        menu.Items.Add("Settings…", null, (_, _) => ShowViewSettings());
        menu.Items.Add("Hotkeys…", null, (_, _) => ShowHotkeysSettings());
        menu.Items.Add(new ToolStripSeparator());

        var theme = new ToolStripMenuItem("Theme") { ForeColor = Theme.Ink };
        ThemeSwitcher.FillSubmenu(theme, _btnOptions);
        menu.Items.Add(theme);

        menu.Show(_btnOptions, new Point(0, _btnOptions.Height));
    }

    private void ShowHelpMenu()
    {
        var menu = Theme.Menu();
        menu.Items.Add("How to record and playback?", null, (_, _) => Msg.Info(
            this,
            "Press Record, perform the clicks and mouse movements you need, then press Stop.\n" +
            "Playback repeats the recording at the same speed."));
        menu.Items.Add("About", null, (_, _) => Msg.Info(
            this,
            "Simple Auto Clicker 3.1.1 — record and playback of mouse actions."));
        menu.Show(_btnHelp, new Point(0, _btnHelp.Height));
    }

    private void ShowMoreMenu()
    {
        var menu = Theme.Menu();
        menu.Items.Add("Clear recording", null, (_, _) =>
        {
            if (_recording || _playing)
            {
                return;
            }

            _events.Clear();
            UpdateState();
        });
        menu.Items.Add(new ToolStripSeparator());
        menu.Items.Add("Settings…", null, (_, _) => ShowViewSettings());
        menu.Items.Add("Hotkeys…", null, (_, _) => ShowHotkeysSettings());
        menu.Show(_btnMore, new Point(0, _btnMore.Height));
    }

    private void ShowViewSettings()
    {
        using var dialog = new AdvancedSettingsDialog(_minimizeWhenPlaying, _minimizeWhenRecording);
        if (dialog.ShowDialog(this) != DialogResult.OK)
        {
            return;
        }

        _minimizeWhenPlaying = dialog.MinimizeWhenPlaying;
        _minimizeWhenRecording = dialog.MinimizeWhenRecording;

        AppSettings.Current.MinimizeWhenPlaying = _minimizeWhenPlaying;
        AppSettings.Current.MinimizeWhenRecording = _minimizeWhenRecording;
        AppSettings.Current.Save();
    }

    private void ShowHotkeysSettings()
    {
        using var dialog = new AdvancedHotkeysDialog(_playbackHotkey, _recordHotkey);
        if (dialog.ShowDialog(this) != DialogResult.OK)
        {
            return;
        }

        int oldPlay = _playbackHotkey;
        int oldRec = _recordHotkey;
        _playbackHotkey = dialog.PlaybackKey;
        _recordHotkey = dialog.RecordKey;

        if (!RegisterHotkeys())
        {
            _playbackHotkey = oldPlay;
            _recordHotkey = oldRec;
            RegisterHotkeys();
            Msg.Warn(this,
                "Those hotkeys are already used by another program. The previous ones were kept.");
            return;
        }

        AppSettings.Current.PlaybackHotkey = _playbackHotkey;
        AppSettings.Current.RecordHotkey = _recordHotkey;
        AppSettings.Current.Save();
    }

    private bool RegisterHotkeys()
    {
        Native.UnregisterHotKey(Handle, HotkeyPlayback);
        Native.UnregisterHotKey(Handle, HotkeyRecord);

        bool ok = true;
        if (_playbackHotkey != 0)
        {
            ok &= Native.RegisterHotKey(Handle, HotkeyPlayback, 0, (uint)_playbackHotkey);
        }

        if (_recordHotkey != 0)
        {
            ok &= Native.RegisterHotKey(Handle, HotkeyRecord, 0, (uint)_recordHotkey);
        }

        return ok;
    }
}

/// <summary>Настройки окна записи: сворачивать ли его во время работы.</summary>
internal sealed class AdvancedSettingsDialog : Form
{
    private readonly ModernCheck _chkPlay;
    private readonly ModernCheck _chkRecord;

    public bool MinimizeWhenPlaying => _chkPlay.Checked;

    public bool MinimizeWhenRecording => _chkRecord.Checked;

    public AdvancedSettingsDialog(bool minimizeWhenPlaying, bool minimizeWhenRecording)
    {
        Text = "View Setting";
        ClientSize = new Size(380, 220);
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        ShowInTaskbar = false;
        StartPosition = FormStartPosition.CenterParent;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F);

        _chkPlay = new ModernCheck
        {
            Text = "Minimized when playing",
            Bounds = new Rectangle(28, 32, 320, 24),
            Checked = minimizeWhenPlaying,
            Surface = Theme.Bg,
        };
        _chkRecord = new ModernCheck
        {
            Text = "Minimized when recording",
            Bounds = new Rectangle(28, 68, 320, 24),
            Checked = minimizeWhenRecording,
            Surface = Theme.Bg,
        };

        var ok = new ModernButton
        {
            Text = "Ok",
            Style = BtnStyle.Primary,
            Bounds = new Rectangle(28, 150, 152, 42),
            DialogResult = DialogResult.OK,
        };
        var cancel = new ModernButton
        {
            Text = "Cancel",
            Style = BtnStyle.Secondary,
            Bounds = new Rectangle(200, 150, 152, 42),
            DialogResult = DialogResult.Cancel,
        };

        Controls.AddRange(new Control[] { _chkPlay, _chkRecord, ok, cancel });
        AcceptButton = ok;
        CancelButton = cancel;

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }
}

/// <summary>Горячие клавиши записи и воспроизведения.</summary>
internal sealed class AdvancedHotkeysDialog : Form
{
    private readonly KeyField _play;
    private readonly KeyField _record;

    public int PlaybackKey => _play.VirtualKey;

    public int RecordKey => _record.VirtualKey;

    public AdvancedHotkeysDialog(int playbackKey, int recordKey)
    {
        Text = "Hotkeys Setting";
        ClientSize = new Size(380, 250);
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        ShowInTaskbar = false;
        StartPosition = FormStartPosition.CenterParent;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F);

        var lblPlay = Caption.Make("Playback / Stop");
        lblPlay.Bounds = new Rectangle(28, 24, 150, 18);
        lblPlay.Font = Theme.F(9F);
        _play = new KeyField(playbackKey) { Bounds = new Rectangle(28, 44, 324, 38) };

        var lblRec = Caption.Make("Record / Stop");
        lblRec.Bounds = new Rectangle(28, 92, 150, 18);
        lblRec.Font = Theme.F(9F);
        _record = new KeyField(recordKey) { Bounds = new Rectangle(28, 112, 324, 38) };

        var ok = new ModernButton
        {
            Text = "Ok",
            Style = BtnStyle.Primary,
            Bounds = new Rectangle(28, 180, 152, 42),
            DialogResult = DialogResult.OK,
        };
        var cancel = new ModernButton
        {
            Text = "Cancel",
            Style = BtnStyle.Secondary,
            Bounds = new Rectangle(200, 180, 152, 42),
            DialogResult = DialogResult.Cancel,
        };

        Controls.AddRange(new Control[] { lblPlay, _play, lblRec, _record, ok, cancel });
        AcceptButton = ok;
        CancelButton = cancel;

        KeyField? active = _play;
        _play.Activated += (_, _) => active = _play;
        _record.Activated += (_, _) => active = _record;
        KeyPreview = true;
        KeyDown += (_, e) =>
        {
            if (e.KeyCode is Keys.Escape or Keys.Enter)
            {
                return;
            }

            active?.CaptureKey(e.KeyCode);
            e.Handled = true;
            e.SuppressKeyPress = true;
        };
        Shown += (_, _) => _play.FocusField();

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }
}
