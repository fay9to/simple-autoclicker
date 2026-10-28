using System.Globalization;
using System.Windows.Forms;

namespace SimpleAutoClicker;

public sealed class MainForm : Form
{
    private const string BaseTitle = "Simple Auto Clicker 3.1.1";
    private const int HotkeyId = 0x0ACE;

    private readonly ModernTextBox _txtHours = new("0");
    private readonly ModernTextBox _txtMins = new("0");
    private readonly ModernTextBox _txtSecs = new("0");
    private readonly ModernTextBox _txtMillis = new("100");

    private readonly ModernSelect _cboButton = new();
    private readonly ModernSelect _cboType = new();

    private readonly ModernRadio _rbRepeat = new();
    private readonly ModernSpin _numRepeat = new();
    private readonly ModernRadio _rbUntilStopped = new();

    private readonly ModernRadio _rbCurrent = new();
    private readonly ModernRadio _rbPicked = new();
    private readonly ModernButton _btnPick = new();
    private readonly ModernTextBox _txtX = new("0");
    private readonly ModernTextBox _txtY = new("0");

    private readonly ModernButton _btnStart = new();
    private readonly ModernButton _btnStop = new();
    private readonly ModernButton _btnHotkey = new();
    private readonly ModernButton _btnRecord = new();
    private readonly ModernButton _btnTheme = new();

    private readonly ClickerEngine _engine = new();
    private readonly ToolTip _tip = new();

    private int _hotkeyVk = 0x75; // F6
    private bool _everStarted;
    private AdvancedForm? _advanced;

    public MainForm()
    {
        Text = BaseTitle;
        ClientSize = new Size(460, 510);
        FormBorderStyle = FormBorderStyle.FixedSingle;
        MaximizeBox = false;
        StartPosition = FormStartPosition.CenterScreen;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F);
        DoubleBuffered = true;

        BuildUi();
        WireEvents();

        _hotkeyVk = AppSettings.Current.Hotkey;
        _btnStop.Enabled = false;
        _txtX.Enabled = false;
        _txtY.Enabled = false;
        _tip.SetToolTip(_btnPick, "Capture the current cursor position");
        _tip.SetToolTip(_btnTheme, "Appearance: system, light or dark");

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }

    protected override void OnShown(EventArgs e)
    {
        base.OnShown(e);
        if (!RegisterCurrentHotkey())
        {
            // при перезапуске предыдущий экземпляр мог ещё держать клавишу
            RetryHotkeyRegistration();
        }
    }

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        _engine.Stop();
        Native.UnregisterHotKey(Handle, HotkeyId);
        if (_advanced is { IsDisposed: false })
        {
            _advanced.Close();
        }

        base.OnFormClosing(e);
    }

    protected override void WndProc(ref Message m)
    {
        if (m.Msg == Native.WM_HOTKEY && m.WParam.ToInt32() == HotkeyId)
        {
            ToggleClicking();
            return;
        }

        base.WndProc(ref m);
    }

    // ---------------------------------------------------------------- UI

    private void BuildUi()
    {
        // ---- Click interval ----
        var cardInterval = new Card("Click interval", new Rectangle(16, 16, 428, 104));
        AddField(cardInterval, _txtHours, "hours", 16);
        AddField(cardInterval, _txtMins, "mins", 118);
        AddField(cardInterval, _txtSecs, "secs", 220);
        AddField(cardInterval, _txtMillis, "milliseconds", 322);

        _btnTheme.Style = BtnStyle.Ghost;
        _btnTheme.Surface = Theme.Card;
        _btnTheme.Icon = Theme.IsDark ? IconKind.Sun : IconKind.Moon;
        _btnTheme.IconSize = 15;
        _btnTheme.SetBounds(388, 8, 26, 26);
        cardInterval.Controls.Add(_btnTheme);

        // ---- Click options ----
        var cardOptions = new Card("Click options", new Rectangle(16, 132, 206, 150));

        var captionButton = Caption.Make("Mouse button", Theme.Card);
        captionButton.Bounds = new Rectangle(14, 38, 178, 16);
        cardOptions.Controls.Add(captionButton);

        _cboButton.AddItems("Left", "Right", "Middle");
        _cboButton.SelectedIndex = 0;
        _cboButton.Surface = Theme.Card;
        _cboButton.SetBounds(14, 56, 178, 32);
        cardOptions.Controls.Add(_cboButton);

        var captionType = Caption.Make("Click type", Theme.Card);
        captionType.Bounds = new Rectangle(14, 96, 178, 16);
        cardOptions.Controls.Add(captionType);

        _cboType.AddItems("Single", "Double");
        _cboType.SelectedIndex = 0;
        _cboType.Surface = Theme.Card;
        _cboType.SetBounds(14, 114, 178, 32);
        cardOptions.Controls.Add(_cboType);

        // ---- Click repeat ----
        var cardRepeat = new Card("Click repeat", new Rectangle(238, 132, 206, 150));

        _rbRepeat.Text = "Repeat";
        _rbRepeat.SetBounds(14, 32, 120, 22);

        _numRepeat.Enabled = false;
        _numRepeat.Surface = Theme.Card;
        _numRepeat.SetBounds(14, 62, 84, 34);

        var captionTimes = Caption.Make("times", Theme.Card);
        captionTimes.Bounds = new Rectangle(104, 70, 60, 18);

        _rbUntilStopped.Text = "Repeat until stopped";
        _rbUntilStopped.SetBounds(14, 108, 186, 22);
        _rbUntilStopped.Checked = true;

        cardRepeat.Controls.AddRange(new Control[]
        {
            _rbRepeat, _numRepeat, captionTimes, _rbUntilStopped,
        });

        // ---- Cursor position ----
        var cardCursor = new Card("Cursor position", new Rectangle(16, 294, 428, 96));

        _rbCurrent.Text = "Current location";
        _rbCurrent.SetBounds(14, 32, 170, 22);
        _rbCurrent.Checked = true;

        _rbPicked.SetBounds(15, 64, 20, 22);

        _btnPick.Text = "Pick location";
        _btnPick.Style = BtnStyle.Ghost;
        _btnPick.Icon = IconKind.Pick;
        _btnPick.IconSize = 16;
        _btnPick.Surface = Theme.Card;
        _btnPick.SetBounds(41, 58, 132, 34);

        var captionX = Caption.Make("X", Theme.Card);
        captionX.Bounds = new Rectangle(184, 66, 14, 20);
        _txtX.TextAlign = HorizontalAlignment.Center;
        _txtX.SetBounds(200, 60, 58, 34);

        var captionY = Caption.Make("Y", Theme.Card);
        captionY.Bounds = new Rectangle(266, 66, 14, 20);
        _txtY.SetBounds(282, 60, 58, 34);

        cardCursor.Controls.AddRange(new Control[]
        {
            _rbCurrent, _rbPicked, _btnPick, captionX, _txtX, captionY, _txtY,
        });

        // ---- Bottom buttons ----
        _btnStart.Text = "Start (F6)";
        _btnStart.Style = BtnStyle.Primary;
        _btnStart.SetBounds(16, 404, 206, 44);

        _btnStop.Text = "Stop (F6)";
        _btnStop.Style = BtnStyle.Danger;
        _btnStop.SetBounds(238, 404, 206, 44);

        _btnHotkey.Text = "Hotkey setting";
        _btnHotkey.Style = BtnStyle.Ghost;
        _btnHotkey.SetBounds(16, 458, 206, 36);

        _btnRecord.Text = "Record & Playback";
        _btnRecord.Style = BtnStyle.Ghost;
        _btnRecord.SetBounds(238, 458, 206, 36);

        Controls.AddRange(new Control[]
        {
            cardInterval, cardOptions, cardRepeat, cardCursor,
            _btnStart, _btnStop, _btnHotkey, _btnRecord,
        });
    }

    private static void AddField(Card card, ModernTextBox field, string caption, int x)
    {
        var label = Caption.Make(caption, Theme.Card);
        label.Bounds = new Rectangle(x, 38, 90, 16);
        label.TextAlign = ContentAlignment.MiddleCenter;
        card.Controls.Add(label);

        field.Surface = Theme.Card;
        field.SetBounds(x, 58, 90, 34);
        card.Controls.Add(field);
    }

    private void WireEvents()
    {
        _btnStart.Click += (_, _) => StartClicking();
        _btnStop.Click += (_, _) => StopClicking();
        _btnHotkey.Click += (_, _) => ChangeHotkey();
        _btnRecord.Click += (_, _) => ShowAdvanced();
        _btnPick.Click += (_, _) => PickLocation();
        _btnTheme.Click += (_, _) => ThemeSwitcher.ShowMenu(_btnTheme);

        foreach (ModernTextBox box in new[] { _txtHours, _txtMins, _txtSecs, _txtMillis, _txtX, _txtY })
        {
            box.MaxLength = 9;
            box.KeyPress += (_, e) =>
            {
                if (!char.IsControl(e.KeyChar) && !char.IsDigit(e.KeyChar))
                {
                    e.Handled = true;
                }
            };
        }

        _rbRepeat.CheckedChanged += (_, _) =>
        {
            _numRepeat.Enabled = _rbRepeat.Checked;
            if (_rbRepeat.Checked)
            {
                _rbUntilStopped.Checked = false;
            }
        };
        _rbUntilStopped.CheckedChanged += (_, _) =>
        {
            if (_rbUntilStopped.Checked)
            {
                _rbRepeat.Checked = false;
                _numRepeat.Enabled = false;
            }
        };

        _rbCurrent.CheckedChanged += (_, _) => UpdateFixedFields();
        _rbPicked.CheckedChanged += (_, _) => UpdateFixedFields();

        _engine.Clicked += _ => SafeInvoke(() =>
        {
            if (_engine.IsRunning && !Text.StartsWith("Running", StringComparison.Ordinal))
            {
                Text = $"Running - {BaseTitle}";
            }
        });
        _engine.Finished += () => SafeInvoke(() =>
        {
            _btnStart.Enabled = true;
            _btnStop.Enabled = false;
            Text = $"Stopped - {BaseTitle}";
        });
    }

    // ---------------------------------------------------------------- actions

    private void UpdateFixedFields()
    {
        bool fixedMode = _rbPicked.Checked;
        _txtX.Enabled = fixedMode;
        _txtY.Enabled = fixedMode;
    }

    private int ReadNumber(ModernTextBox box)
    {
        if (int.TryParse(box.Text.Trim(), NumberStyles.Integer, CultureInfo.InvariantCulture, out int v))
        {
            return Math.Max(0, v);
        }

        return 0;
    }

    private ClickSettings BuildSettings()
    {
        long total = ReadNumber(_txtHours) * 3600000L
                     + ReadNumber(_txtMins) * 60000L
                     + ReadNumber(_txtSecs) * 1000L
                     + ReadNumber(_txtMillis);

        if (total <= 0)
        {
            total = 1; // 0 мс = «как можно быстрее»
        }

        return new ClickSettings
        {
            IntervalMs = (int)Math.Min(total, int.MaxValue),
            Button = _cboButton.SelectedIndex switch
            {
                1 => MouseButtonKind.Right,
                2 => MouseButtonKind.Middle,
                _ => MouseButtonKind.Left,
            },
            DoubleClick = _cboType.SelectedIndex == 1,
            RepeatForever = _rbUntilStopped.Checked,
            RepeatCount = _numRepeat.Value,
            FixedPosition = _rbPicked.Checked,
            X = ReadNumber(_txtX),
            Y = ReadNumber(_txtY),
        };
    }

    private void StartClicking()
    {
        if (_engine.IsRunning)
        {
            return;
        }

        _everStarted = true;
        _btnStart.Enabled = false;
        _btnStop.Enabled = true;
        Text = $"Running - {BaseTitle}";
        _engine.Start(BuildSettings());
    }

    private void StopClicking()
    {
        _engine.Stop();
        if (!_engine.IsRunning)
        {
            _btnStart.Enabled = true;
            _btnStop.Enabled = false;
            Text = _everStarted ? $"Stopped - {BaseTitle}" : BaseTitle;
        }
    }

    private void ToggleClicking()
    {
        if (_engine.IsRunning)
        {
            StopClicking();
        }
        else
        {
            StartClicking();
        }
    }

    private void ChangeHotkey()
    {
        using var dialog = new HotkeyDialog(_hotkeyVk);
        if (dialog.ShowDialog(this) != DialogResult.OK)
        {
            return;
        }

        int previous = _hotkeyVk;
        _hotkeyVk = dialog.VirtualKey;
        if (!RegisterCurrentHotkey())
        {
            _hotkeyVk = previous;
            RegisterCurrentHotkey();
            Msg.Warn(this,
                $"Could not assign {KeysUtil.Name(dialog.VirtualKey)} — " +
                "another program may already use this key.");
            return;
        }

        AppSettings.Current.Hotkey = _hotkeyVk;
        AppSettings.Current.Save();
        UpdateHotkeyButtonText();
    }

    private bool RegisterCurrentHotkey()
    {
        Native.UnregisterHotKey(Handle, HotkeyId);
        bool ok = Native.RegisterHotKey(Handle, HotkeyId, 0, (uint)_hotkeyVk);
        UpdateHotkeyButtonText();
        return ok;
    }

    /// <summary>Пара попыток на случай, если клавишу ещё держит предыдущий экземпляр.</summary>
    private void RetryHotkeyRegistration()
    {
        int attempts = 0;
        var timer = new System.Windows.Forms.Timer { Interval = 250 };
        timer.Tick += (_, _) =>
        {
            attempts++;
            if (IsDisposed || attempts > 12 || RegisterCurrentHotkey())
            {
                timer.Stop();
                timer.Dispose();
            }
        };
        timer.Start();
    }

    private void UpdateHotkeyButtonText()
    {
        string name = KeysUtil.Name(_hotkeyVk);
        _btnStart.Text = $"Start ({name})";
        _btnStop.Text = $"Stop ({name})";
    }

    private void PickLocation()
    {
        using var picker = new PickLocationForm(3);
        picker.ShowDialog(this);

        var point = picker.Result;
        if (point is null)
        {
            return;
        }

        _txtX.Text = point.Value.X.ToString(CultureInfo.InvariantCulture);
        _txtY.Text = point.Value.Y.ToString(CultureInfo.InvariantCulture);
        _rbPicked.Checked = true;
    }

    private void ShowAdvanced()
    {
        if (_advanced is { IsDisposed: false })
        {
            _advanced.Activate();
            return;
        }

        _advanced = new AdvancedForm();
        _advanced.FormClosed += (_, _) =>
        {
            _advanced = null;
            Show();
            Activate();
        };

        // как в оригинале: главное окно убирается, пока открыто окно записи
        Hide();
        _advanced.Show();
    }

    private void SafeInvoke(Action action)
    {
        if (IsDisposed)
        {
            return;
        }

        if (InvokeRequired)
        {
            try
            {
                BeginInvoke(action);
            }
            catch (ObjectDisposedException)
            {
                // окно уже закрыто
            }
        }
        else
        {
            action();
        }
    }
}
