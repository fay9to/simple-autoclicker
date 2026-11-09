using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace SimpleAutoClicker;

internal static class KeysUtil
{
    public static string Name(int vk)
    {
        if (vk == 0)
        {
            return "None";
        }

        Keys key = (Keys)vk;
        if (key >= Keys.D0 && key <= Keys.D9)
        {
            return ((char)('0' + (key - Keys.D0))).ToString();
        }

        return key.ToString();
    }
}

/// <summary>Поле, которое запоминает нажатую клавишу вместо ввода текста.</summary>
internal sealed class KeyField : Panel
{
    private readonly TextBox _box = new();
    private Color _surface = Theme.Bg;
    private bool _focused;

    public int VirtualKey { get; private set; }

    public Color Surface
    {
        get => _surface;
        set
        {
            _surface = value;
            BackColor = value;
            Invalidate();
        }
    }

    public KeyField(int initial)
    {
        DoubleBuffered = true;
        ResizeRedraw = true;
        BackColor = Theme.Bg;
        Height = Ui.S(38);
        Padding = new Padding(Ui.S(10), Ui.S(8), Ui.S(10), Ui.S(8));

        _box.BorderStyle = BorderStyle.None;
        _box.ReadOnly = true;
        _box.BackColor = Theme.Card;
        _box.ForeColor = Theme.Ink;
        _box.Font = Theme.F(10.5F, FontStyle.Bold);
        _box.TextAlign = HorizontalAlignment.Center;
        _box.Cursor = Cursors.Hand;
        _box.Dock = DockStyle.Fill;
        _box.ShortcutsEnabled = false;

        VirtualKey = initial;
        _box.Text = KeysUtil.Name(initial);

        _box.GotFocus += (_, _) =>
        {
            _focused = true;
            _box.SelectionStart = _box.TextLength;
            _box.SelectionLength = 0;
            Invalidate();
        };
        _box.LostFocus += (_, _) => { _focused = false; Invalidate(); };
        _box.KeyDown += OnKeyDown;
        _box.MouseDown += (_, _) =>
        {
            _box.Focus();
            Activated?.Invoke(this, EventArgs.Empty);
        };

        Controls.Add(_box);
    }

    /// <summary>Пользователь выбрал это поле (клик или фокус).</summary>
    public event EventHandler? Activated;

    /// <summary>Переносит фокус внутрь поля, чтобы сразу можно было нажимать клавишу.</summary>
    public void FocusField() => _box.Focus();

    /// <summary>Запоминает клавишу, нажатую где угодно в диалоге.</summary>
    public void CaptureKey(Keys key)
    {
        if (key is Keys.ControlKey or Keys.ShiftKey or Keys.Menu or Keys.LWin or Keys.RWin
            or Keys.None or Keys.ProcessKey or Keys.Tab)
        {
            return;
        }

        VirtualKey = (int)key;
        _box.Text = KeysUtil.Name(VirtualKey);
    }

    private void OnKeyDown(object? sender, KeyEventArgs e)
    {
        e.Handled = true;
        e.SuppressKeyPress = true;
        CaptureKey(e.KeyCode);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        PaintHelper.Fill(e.Graphics, this, _surface);
        e.Graphics.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
        var rect = new Rectangle(0, 0, Width - 1, Height - 1);
        using var path = Theme.Round(rect, Ui.S(9));
        using var fill = new SolidBrush(Theme.Field);
        e.Graphics.FillPath(fill, path);
        using var pen = new Pen(_focused ? Theme.BorderFocus : Theme.Border, _focused ? Ui.Sf(1.6f) : 1f);
        e.Graphics.DrawPath(pen, path);
        base.OnPaint(e);
    }
}

/// <summary>Диалог «Hotkey Setting»: одна горячая клавиша Start/Stop.</summary>
internal sealed class HotkeyDialog : Form
{
    private readonly KeyField _field;

    public int VirtualKey => _field.VirtualKey;

    public HotkeyDialog(int current)
    {
        Text = "Hotkey Setting";
        ClientSize = new Size(380, 210);
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        ShowInTaskbar = false;
        StartPosition = FormStartPosition.CenterParent;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F);

        var caption = Caption.Make("Start / Stop");
        caption.Bounds = new Rectangle(28, 26, 160, 18);
        caption.Font = Theme.F(9F);

        _field = new KeyField(current) { Bounds = new Rectangle(28, 46, 324, 38) };

        var ok = new ModernButton
        {
            Text = "Ok",
            Style = BtnStyle.Primary,
            Bounds = new Rectangle(28, 140, 152, 42),
            DialogResult = DialogResult.OK,
        };
        var cancel = new ModernButton
        {
            Text = "Cancel",
            Style = BtnStyle.Secondary,
            Bounds = new Rectangle(200, 140, 152, 42),
            DialogResult = DialogResult.Cancel,
        };

        Controls.AddRange(new Control[] { caption, _field, ok, cancel });
        AcceptButton = ok;
        CancelButton = cancel;
        KeyPreview = true;
        KeyDown += (_, e) =>
        {
            if (e.KeyCode is Keys.Escape or Keys.Enter)
            {
                return;
            }

            _field.CaptureKey(e.KeyCode);
            e.Handled = true;
            e.SuppressKeyPress = true;
        };
        Shown += (_, _) => _field.FocusField();

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }
}

/// <summary>Небольшое окно с обратным отсчётом: за это время надо увести мышь в нужную точку.</summary>
internal sealed class PickLocationForm : Form
{
    private readonly System.Windows.Forms.Timer _timer = new();
    private int _secondsLeft;
    private readonly Label _number = new();

    public Point? Result { get; private set; }

    public PickLocationForm(int seconds)
    {
        _secondsLeft = Math.Max(1, seconds);
        Text = "Pick location";
        ClientSize = new Size(360, 130);
        FormBorderStyle = FormBorderStyle.FixedToolWindow;
        StartPosition = FormStartPosition.CenterScreen;
        TopMost = true;
        ShowInTaskbar = false;
        BackColor = Theme.Card;
        Font = Theme.F(9.5F);

        var caption = new Label
        {
            Text = "Point the cursor at the spot you need",
            Bounds = new Rectangle(16, 18, 328, 22),
            TextAlign = ContentAlignment.MiddleCenter,
            ForeColor = Theme.Muted,
            BackColor = Theme.Card,
        };

        _number.Text = _secondsLeft.ToString();
        _number.Bounds = new Rectangle(16, 46, 328, 60);
        _number.TextAlign = ContentAlignment.MiddleCenter;
        _number.Font = Theme.F(26F, FontStyle.Bold);
        _number.ForeColor = Theme.Accent;
        _number.BackColor = Theme.Card;

        Controls.AddRange(new Control[] { caption, _number });

        _timer.Interval = 1000;
        _timer.Tick += (_, _) =>
        {
            _secondsLeft--;
            if (_secondsLeft > 0)
            {
                _number.Text = _secondsLeft.ToString();
                return;
            }

            _timer.Stop();
            Native.GetCursorPos(out var p);
            Result = new Point(p.X, p.Y);
            Close();
        };
        Shown += (_, _) => _timer.Start();

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }
}

/// <summary>Низкоуровневый хук мыши — используется записью кликов и движений.</summary>
internal sealed class MouseHook : IDisposable
{
    private Native.LowLevelMouseProc? _proc;
    private IntPtr _hook;

    public event Action<MouseButtonKind, bool, int, int>? MouseEvent;

    public event Action<int, int>? MouseMoved;

    public bool IsActive => _hook != IntPtr.Zero;

    public bool Start()
    {
        Stop();

        _proc = Callback;
        _hook = Native.SetWindowsHookEx(Native.WH_MOUSE_LL, _proc, Native.GetModuleHandle(null), 0);
        return _hook != IntPtr.Zero;
    }

    public void Stop()
    {
        if (_hook == IntPtr.Zero)
        {
            return;
        }

        Native.UnhookWindowsHookEx(_hook);
        _hook = IntPtr.Zero;
        _proc = null;
    }

    private IntPtr Callback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            int msg = wParam.ToInt32();
            var data = Marshal.PtrToStructure<Native.MSLLHOOKSTRUCT>(lParam);

            if (msg == Native.WM_MOUSEMOVE)
            {
                MouseMoved?.Invoke(data.pt.X, data.pt.Y);
            }
            else
            {
                (MouseButtonKind kind, bool down, bool known) = msg switch
                {
                    Native.WM_LBUTTONDOWN => (MouseButtonKind.Left, true, true),
                    Native.WM_LBUTTONUP => (MouseButtonKind.Left, false, true),
                    Native.WM_RBUTTONDOWN => (MouseButtonKind.Right, true, true),
                    Native.WM_RBUTTONUP => (MouseButtonKind.Right, false, true),
                    Native.WM_MBUTTONDOWN => (MouseButtonKind.Middle, true, true),
                    Native.WM_MBUTTONUP => (MouseButtonKind.Middle, false, true),
                    _ => (MouseButtonKind.Left, false, false),
                };

                if (known)
                {
                    MouseEvent?.Invoke(kind, down, data.pt.X, data.pt.Y);
                }
            }
        }

        return Native.CallNextHookEx(_hook, nCode, wParam, lParam);
    }

    public void Dispose() => Stop();
}

/// <summary>Записанное событие мыши.</summary>
internal sealed class RecordedEvent
{
    public bool Move { get; init; }

    public MouseButtonKind Button { get; init; }

    public bool Down { get; init; }

    public int X { get; init; }

    public int Y { get; init; }

    public long TimeMs { get; init; }
}

/// <summary>Сообщение в оформлении приложения: штатный MessageBox всегда светлый.</summary>
internal static class Msg
{
    public static void Info(IWin32Window? owner, string text, string title = "Simple Auto Clicker")
    {
        using var dialog = new MessageDialog(title, text, false, false, "Ok");
        dialog.ShowDialog(owner);
    }

    public static void Warn(IWin32Window? owner, string text, string title = "Simple Auto Clicker")
    {
        using var dialog = new MessageDialog(title, text, true, false, "Ok");
        dialog.ShowDialog(owner);
    }

    public static bool Confirm(IWin32Window? owner, string text, string okText, string title = "Simple Auto Clicker")
    {
        using var dialog = new MessageDialog(title, text, false, true, okText);
        return dialog.ShowDialog(owner) == DialogResult.OK;
    }
}

internal sealed class MessageDialog : Form
{
    public MessageDialog(string title, string text, bool warning, bool confirm, string okText)
    {
        Text = title;
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        ShowInTaskbar = false;
        StartPosition = FormStartPosition.CenterParent;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F);
        DoubleBuffered = true;

        const int width = 404;

        var bar = new Panel
        {
            BackColor = warning ? Theme.Danger : Theme.Accent,
            Bounds = new Rectangle(0, 0, width, 4),
        };

        var body = new Label
        {
            Text = text,
            AutoSize = false,
            ForeColor = Theme.Ink,
            BackColor = Theme.Bg,
            Font = Theme.F(10F),
            Bounds = new Rectangle(26, 24, width - 52, 20),
        };

        Size measured = TextRenderer.MeasureText(
            text,
            body.Font,
            new Size(width - 52, 0),
            TextFormatFlags.WordBreak | TextFormatFlags.NoPrefix);
        body.Height = Math.Max(20, measured.Height + 4);

        int buttonsTop = body.Bottom + 22;
        int okWidth = confirm ? 132 : 140;

        var ok = new ModernButton
        {
            Text = okText,
            Style = BtnStyle.Primary,
            Bounds = new Rectangle(width - 26 - okWidth, buttonsTop, okWidth, 42),
            DialogResult = DialogResult.OK,
        };

        Controls.Add(bar);
        Controls.Add(body);
        Controls.Add(ok);
        AcceptButton = ok;

        if (confirm)
        {
            var cancel = new ModernButton
            {
                Text = "Cancel",
                Style = BtnStyle.Secondary,
                Bounds = new Rectangle(width - 26 - okWidth - 144, buttonsTop, 132, 42),
                DialogResult = DialogResult.Cancel,
            };
            Controls.Add(cancel);
            CancelButton = cancel;
        }

        ClientSize = new Size(width, buttonsTop + 42 + 24);

        Theme.StyleWindow(this);
        Ui.Apply(this);
    }
}
