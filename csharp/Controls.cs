using System.Drawing.Drawing2D;
using System.Globalization;
using System.Windows.Forms;

namespace SimpleAutoClicker;

/// <summary>Общий помощник: залить фон контрола его «поверхностью».</summary>
internal static class PaintHelper
{
    public static void Fill(Graphics g, Control c, Color color)
    {
        using var brush = new SolidBrush(color);
        g.FillRectangle(brush, c.ClientRectangle);
    }
}

/// <summary>Белая «карточка» со скруглёнными углами и заголовком.</summary>
internal sealed class Card : Panel
{
    private readonly string _title;

    public Card(string title, Rectangle bounds)
    {
        _title = title;
        Bounds = bounds;
        BackColor = Theme.Bg;
        DoubleBuffered = true;
        ResizeRedraw = true;
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, Theme.Bg);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        var rect = new Rectangle(Ui.S(1), Ui.S(1), Width - Ui.S(2), Height - Ui.S(2));
        using var path = Theme.Round(rect, Ui.S(12));
        using (var fill = new SolidBrush(Theme.Card))
        {
            g.FillPath(fill, path);
        }

        using (var pen = new Pen(Theme.Border))
        {
            g.DrawPath(pen, path);
        }

        if (_title.Length > 0)
        {
            using var font = Theme.F(9.5F, FontStyle.Bold);
            TextRenderer.DrawText(
                g,
                _title,
                font,
                new Point(Ui.S(16), Ui.S(11)),
                Theme.Ink,
                TextFormatFlags.NoPrefix);
        }
    }
}

internal enum BtnStyle
{
    Primary,
    Secondary,
    Danger,
    Ghost,
}

internal sealed class ModernButton : Button
{
    private Color _surface = Theme.Bg;
    private bool _hover;
    private bool _down;
    private BtnStyle _style = BtnStyle.Secondary;
    private IconKind _icon = IconKind.None;
    private int _iconSize = 20;
    private Color? _iconColor;

    public BtnStyle Style
    {
        get => _style;
        set
        {
            _style = value;
            Invalidate();
        }
    }

    public IconKind Icon
    {
        get => _icon;
        set
        {
            _icon = value;
            Invalidate();
        }
    }

    public int IconSize
    {
        get => _iconSize;
        set
        {
            _iconSize = value;
            Invalidate();
        }
    }

    /// <summary>Цвет иконки; null — цвет текста кнопки.</summary>
    public Color? IconColor
    {
        get => _iconColor;
        set
        {
            _iconColor = value;
            Invalidate();
        }
    }

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

    public ModernButton()
    {
        SetStyle(
            ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint |
            ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw,
            true);
        FlatStyle = FlatStyle.Flat;
        FlatAppearance.BorderSize = 0;
        BackColor = Theme.Bg;
        Font = Theme.F(9.5F, FontStyle.Bold);
        Cursor = Cursors.Hand;
        TabStop = false;
        UseVisualStyleBackColor = false;
    }

    protected override void OnMouseEnter(EventArgs e)
    {
        _hover = true;
        Invalidate();
        base.OnMouseEnter(e);
    }

    protected override void OnMouseLeave(EventArgs e)
    {
        _hover = false;
        _down = false;
        Invalidate();
        base.OnMouseLeave(e);
    }

    protected override void OnMouseDown(MouseEventArgs e)
    {
        _down = true;
        Invalidate();
        base.OnMouseDown(e);
    }

    protected override void OnMouseUp(MouseEventArgs e)
    {
        _down = false;
        Invalidate();
        base.OnMouseUp(e);
    }

    protected override void OnEnabledChanged(EventArgs e)
    {
        Invalidate();
        base.OnEnabledChanged(e);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, _surface);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        (Color back, Color fore, Color border) = ResolveColors();
        var rect = new Rectangle(0, 0, Width - 1, Height - 1);
        using var path = Theme.Round(rect, Ui.S(10));

        using (var fill = new SolidBrush(back))
        {
            g.FillPath(fill, path);
        }

        if (border.A > 0)
        {
            using var pen = new Pen(border);
            g.DrawPath(pen, path);
        }

        bool hasIcon = Icon != IconKind.None;
        string text = Text;
        Color iconColor = IconColor ?? fore;

        if (hasIcon && text.Length == 0)
        {
            int size = Ui.S(IconSize);
            var iconRect = new Rectangle((Width - size) / 2, (Height - size) / 2, size, size);
            Theme.DrawIcon(g, iconRect, Icon, iconColor);
            return;
        }

        if (hasIcon)
        {
            int size = Ui.S(IconSize);
            int gap = Ui.S(10);
            Size textSize = TextRenderer.MeasureText(text, Font, new Size(int.MaxValue, int.MaxValue),
                TextFormatFlags.NoPrefix);
            int total = size + gap + textSize.Width;
            int x = (Width - total) / 2;
            var iconRect = new Rectangle(x, (Height - size) / 2, size, size);
            Theme.DrawIcon(g, iconRect, Icon, iconColor);
            TextRenderer.DrawText(
                g, text, Font,
                new Rectangle(x + size + gap, 0, textSize.Width + 2, Height),
                fore,
                TextFormatFlags.VerticalCenter | TextFormatFlags.NoPrefix);
            return;
        }

        TextRenderer.DrawText(
            g, text, Font,
            new Rectangle(0, 0, Width, Height),
            fore,
            TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter | TextFormatFlags.NoPrefix);
    }

    private (Color Back, Color Fore, Color Border) ResolveColors()
    {
        if (!Enabled)
        {
            return Style switch
            {
                BtnStyle.Primary => (Theme.DisabledSurface, Theme.OnAccent, Color.Transparent),
                BtnStyle.Ghost => (Theme.Soft, Theme.DisabledInk, Color.Transparent),
                _ => (Theme.Card, Theme.DisabledInk, Theme.Border),
            };
        }

        return Style switch
        {
            BtnStyle.Primary => (
                _down ? Theme.AccentDown : _hover ? Theme.AccentHover : Theme.Accent,
                Theme.OnAccent,
                Color.Transparent),
            BtnStyle.Danger => (
                _down ? Theme.DangerDown : _hover ? Theme.DangerHover : Theme.DangerSoft,
                Theme.Danger,
                Color.Transparent),
            BtnStyle.Ghost => (
                _down || _hover ? Theme.SoftHover : Theme.Soft,
                Theme.Ink,
                Color.Transparent),
            _ => (
                _down ? Theme.Soft : _hover ? Theme.HoverCard : Theme.Card,
                Theme.Ink,
                Theme.Border),
        };
    }
}

internal sealed class ModernRadio : RadioButton
{
    private Color _surface = Theme.Card;

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

    public ModernRadio()
    {
        SetStyle(
            ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint |
            ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw,
            true);
        AutoSize = false;
        Font = Theme.F(9.5F);
        ForeColor = Theme.Ink;
        BackColor = Theme.Card;
        Cursor = Cursors.Hand;
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, _surface);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        int d = Ui.S(16);
        int top = (Height - d) / 2;
        var circle = new Rectangle(Ui.S(1), top, d, d);

        using (var fill = new SolidBrush(Enabled ? Theme.Field : Theme.Soft))
        {
            g.FillEllipse(fill, circle);
        }

        using (var pen = new Pen(Checked ? Theme.Accent : Theme.ControlBorder, Ui.Sf(1.6f)))
        {
            g.DrawEllipse(pen, circle);
        }

        if (Checked)
        {
            int dot = Ui.S(8);
            using var brush = new SolidBrush(Theme.Accent);
            g.FillEllipse(brush, circle.X + (d - dot) / 2, circle.Y + (d - dot) / 2, dot, dot);
        }

        if (Text.Length > 0)
        {
            TextRenderer.DrawText(
                g, Text, Font,
                new Rectangle(circle.Right + Ui.S(8), 0, Width - circle.Right - Ui.S(8), Height),
                Enabled ? Theme.Ink : Theme.Muted,
                TextFormatFlags.VerticalCenter | TextFormatFlags.NoPrefix);
        }
    }
}

internal sealed class ModernCheck : CheckBox
{
    private Color _surface = Theme.Card;

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

    public ModernCheck()
    {
        SetStyle(
            ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint |
            ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw,
            true);
        AutoSize = false;
        Font = Theme.F(9.5F);
        BackColor = Theme.Card;
        Cursor = Cursors.Hand;
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, _surface);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        int d = Ui.S(18);
        int top = (Height - d) / 2;
        var box = new Rectangle(Ui.S(1), top, d, d);
        using var path = Theme.Round(box, Ui.S(5));

        using (var fill = new SolidBrush(Checked ? Theme.Accent : Enabled ? Theme.Field : Theme.Soft))
        {
            g.FillPath(fill, path);
        }

        using (var pen = new Pen(Checked ? Theme.Accent : Theme.ControlBorder, Ui.Sf(1.6f)))
        {
            g.DrawPath(pen, path);
        }

        if (Checked)
        {
            using var pen = new Pen(Theme.OnAccent, Ui.Sf(2.2f))
            {
                StartCap = LineCap.Round,
                EndCap = LineCap.Round,
            };
            g.DrawLines(pen, new[]
            {
                new PointF(box.Left + d * 0.24f, box.Top + d * 0.52f),
                new PointF(box.Left + d * 0.44f, box.Top + d * 0.72f),
                new PointF(box.Left + d * 0.78f, box.Top + d * 0.30f),
            });
        }

        TextRenderer.DrawText(
            g, Text, Font,
            new Rectangle(box.Right + Ui.S(10), 0, Width - box.Right - Ui.S(10), Height),
            Enabled ? Theme.Ink : Theme.Muted,
            TextFormatFlags.VerticalCenter | TextFormatFlags.NoPrefix);
    }
}

/// <summary>Текстовое поле в скруглённой рамке.</summary>
internal sealed class ModernTextBox : Panel
{
    private readonly TextBox _box = new();
    private Color _surface = Theme.Card;
    private bool _focused;
    private bool _readOnlyLook;

    public ModernTextBox(string text)
    {
        DoubleBuffered = true;
        ResizeRedraw = true;
        BackColor = Theme.Card;
        Height = Ui.S(34);
        Padding = new Padding(Ui.S(10), Ui.S(8), Ui.S(10), Ui.S(8));

        _box.BorderStyle = BorderStyle.None;
        _box.BackColor = Theme.Field;
        _box.ForeColor = Theme.Ink;
        _box.Font = Theme.F(10F);
        _box.TextAlign = HorizontalAlignment.Center;
        _box.Text = text;
        _box.Dock = DockStyle.Fill;
        _box.GotFocus += (_, _) => { _focused = true; Invalidate(); };
        _box.LostFocus += (_, _) => { _focused = false; Invalidate(); };
        Controls.Add(_box);
    }

    public TextBox Inner => _box;

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

    public bool ReadOnlyLook
    {
        get => _readOnlyLook;
        set
        {
            _readOnlyLook = value;
            Invalidate();
        }
    }

    [System.Diagnostics.CodeAnalysis.AllowNull]
    public override string Text
    {
        get => _box.Text;
        set => _box.Text = value ?? string.Empty;
    }

    public new bool Enabled
    {
        get => base.Enabled;
        set
        {
            base.Enabled = value;
            _box.Enabled = value;
            _box.BackColor = value ? Theme.Field : Theme.Soft;
            Invalidate();
        }
    }

    public int MaxLength
    {
        get => _box.MaxLength;
        set => _box.MaxLength = value;
    }

    public HorizontalAlignment TextAlign
    {
        get => _box.TextAlign;
        set => _box.TextAlign = value;
    }

    public new event KeyPressEventHandler? KeyPress
    {
        add => _box.KeyPress += value;
        remove => _box.KeyPress -= value;
    }

    public new event KeyEventHandler? KeyDown
    {
        add => _box.KeyDown += value;
        remove => _box.KeyDown -= value;
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, _surface);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        bool off = !Enabled;
        var rect = new Rectangle(0, 0, Width - 1, Height - 1);
        using var path = Theme.Round(rect, Ui.S(9));
        using (var fill = new SolidBrush(off || ReadOnlyLook ? Theme.Soft : Theme.Field))
        {
            g.FillPath(fill, path);
        }

        using var pen = new Pen(_focused ? Theme.BorderFocus : Theme.Border, _focused ? Ui.Sf(1.6f) : 1f);
        g.DrawPath(pen, path);
        base.OnPaint(e);
    }
}

/// <summary>Числовое поле со стрелками в той же рамке, что и остальные поля.</summary>
internal sealed class ModernSpin : Panel
{
    private Color _surface = Theme.Card;
    private bool _hoverUp;
    private bool _hoverDown;
    private bool _focused;
    private string _edit = string.Empty;
    private bool _editing;
    private int _value = 1;
    private int _min = 1;
    private int _max = 1000000;

    public int Value
    {
        get => _value;
        set
        {
            int clamped = Math.Clamp(value, _min, _max);
            if (clamped == _value)
            {
                return;
            }

            _value = clamped;
            Invalidate();
        }
    }

    public int Minimum
    {
        get => _min;
        set
        {
            _min = value;
            Value = _value;
        }
    }

    public int Maximum
    {
        get => _max;
        set
        {
            _max = value;
            Value = _value;
        }
    }

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

    public ModernSpin()
    {
        SetStyle(
            ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint |
            ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw |
            ControlStyles.Selectable,
            true);
        BackColor = Theme.Card;
        Font = Theme.F(10F);
        Height = Ui.S(34);
        TabStop = true;
        Cursor = Cursors.IBeam;
    }

    private bool InArrows(int x) => x >= Width - Ui.S(30);

    protected override bool IsInputKey(Keys keyData) => true;

    protected override void OnMouseMove(MouseEventArgs e)
    {
        bool up = InArrows(e.X) && e.Y < Height / 2;
        bool down = InArrows(e.X) && e.Y >= Height / 2;
        if (up != _hoverUp || down != _hoverDown)
        {
            _hoverUp = up;
            _hoverDown = down;
            Cursor = InArrows(e.X) ? Cursors.Hand : Cursors.IBeam;
            Invalidate();
        }

        base.OnMouseMove(e);
    }

    protected override void OnMouseLeave(EventArgs e)
    {
        _hoverUp = _hoverDown = false;
        Cursor = Cursors.IBeam;
        Invalidate();
        base.OnMouseLeave(e);
    }

    protected override void OnMouseDown(MouseEventArgs e)
    {
        Focus();

        if (InArrows(e.X))
        {
            if (e.Y < Height / 2)
            {
                Value = _value + 1;
            }
            else
            {
                Value = _value - 1;
            }

            return;
        }

        base.OnMouseDown(e);
    }

    protected override void OnEnter(EventArgs e)
    {
        _focused = true;
        _edit = string.Empty;
        _editing = false;
        Invalidate();
        base.OnEnter(e);
    }

    protected override void OnLeave(EventArgs e)
    {
        _focused = false;
        _editing = false;
        _edit = string.Empty;
        Invalidate();
        base.OnLeave(e);
    }

    protected override void OnKeyPress(KeyPressEventArgs e)
    {
        if (char.IsDigit(e.KeyChar))
        {
            if (!_editing)
            {
                _edit = string.Empty;
                _editing = true;
            }

            if (_edit.Length < 9)
            {
                _edit += e.KeyChar;
            }

            if (int.TryParse(_edit, out int parsed))
            {
                _value = Math.Clamp(parsed, _min, _max);
            }

            e.Handled = true;
            Invalidate();
            return;
        }

        if (e.KeyChar == '\b' && _editing)
        {
            _edit = _edit.Length > 1 ? _edit[..^1] : string.Empty;
            _value = _edit.Length > 0 && int.TryParse(_edit, out int parsed)
                ? Math.Clamp(parsed, _min, _max)
                : _min;
            e.Handled = true;
            Invalidate();
            return;
        }

        if (e.KeyChar == '\r')
        {
            _editing = false;
            _edit = string.Empty;
            Invalidate();
        }

        base.OnKeyPress(e);
    }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        switch (e.KeyCode)
        {
            case Keys.Up:
                Value = _value + 1;
                e.Handled = true;
                Invalidate();
                return;
            case Keys.Down:
                Value = _value - 1;
                e.Handled = true;
                Invalidate();
                return;
        }

        base.OnKeyDown(e);
    }

    protected override void OnMouseWheel(MouseEventArgs e)
    {
        if (_focused)
        {
            Value = _value + (e.Delta > 0 ? 1 : -1);
            Invalidate();
            return;
        }

        base.OnMouseWheel(e);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, _surface);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        var rect = new Rectangle(0, 0, Width - 1, Height - 1);
        using var path = Theme.Round(rect, Ui.S(9));
        using (var fill = new SolidBrush(Enabled ? Theme.Field : Theme.Soft))
        {
            g.FillPath(fill, path);
        }

        using (var pen = new Pen(_focused ? Theme.BorderFocus : Theme.Border, _focused ? Ui.Sf(1.6f) : 1f))
        {
            g.DrawPath(pen, path);
        }

        string text = _editing && _edit.Length > 0 ? _edit : _value.ToString(CultureInfo.InvariantCulture);
        TextRenderer.DrawText(
            g,
            text,
            Font,
            new Rectangle(Ui.S(8), 0, Width - Ui.S(36), Height),
            Enabled ? Theme.Ink : Theme.Muted,
            TextFormatFlags.VerticalCenter | TextFormatFlags.Right | TextFormatFlags.NoPrefix);

        int cx = Width - Ui.S(15);
        int w = Ui.S(5);
        int h = Ui.S(3);
        var upColor = Enabled ? (_hoverUp ? Theme.Accent : Theme.Muted) : Theme.DisabledInk;
        var downColor = Enabled ? (_hoverDown ? Theme.Accent : Theme.Muted) : Theme.DisabledInk;

        using (var up = new SolidBrush(upColor))
        {
            g.FillPolygon(up, new[]
            {
                new PointF(cx, Height / 2f - Ui.S(6)),
                new PointF(cx - w, Height / 2f - Ui.S(6) + h),
                new PointF(cx + w, Height / 2f - Ui.S(6) + h),
            });
        }

        using (var down = new SolidBrush(downColor))
        {
            g.FillPolygon(down, new[]
            {
                new PointF(cx, Height / 2f + Ui.S(6)),
                new PointF(cx - w, Height / 2f + Ui.S(6) - h),
                new PointF(cx + w, Height / 2f + Ui.S(6) - h),
            });
        }
    }
}

/// <summary>Аккуратный выпадающий список: рисуем сами, варианты показываем меню.</summary>
internal sealed class ModernSelect : Panel
{
    private readonly List<string> _items = new();
    private Color _surface = Theme.Card;
    private bool _hover;
    private bool _open;
    private int _selectedIndex = -1;

    public IReadOnlyList<string> Items => _items;

    public int SelectedIndex
    {
        get => _selectedIndex;
        set
        {
            if (value < -1 || value >= _items.Count || value == _selectedIndex)
            {
                return;
            }

            _selectedIndex = value;
            Invalidate();
            SelectedIndexChanged?.Invoke(this, EventArgs.Empty);
        }
    }

    public string SelectedItem => _selectedIndex >= 0 && _selectedIndex < _items.Count
        ? _items[_selectedIndex]
        : string.Empty;

    public event EventHandler? SelectedIndexChanged;

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

    public ModernSelect()
    {
        DoubleBuffered = true;
        ResizeRedraw = true;
        BackColor = Theme.Card;
        Font = Theme.F(9.5F);
        Cursor = Cursors.Hand;
        Height = Ui.S(32);
    }

    public void AddItems(params string[] items)
    {
        _items.AddRange(items);
        if (_selectedIndex < 0 && _items.Count > 0)
        {
            _selectedIndex = 0;
        }

        Invalidate();
    }

    public void ClearItems()
    {
        _items.Clear();
        _selectedIndex = -1;
        Invalidate();
    }

    protected override void OnMouseEnter(EventArgs e)
    {
        _hover = true;
        Invalidate();
        base.OnMouseEnter(e);
    }

    protected override void OnMouseLeave(EventArgs e)
    {
        _hover = false;
        Invalidate();
        base.OnMouseLeave(e);
    }

    protected override void OnMouseDown(MouseEventArgs e)
    {
        base.OnMouseDown(e);
        if (!Enabled)
        {
            return;
        }

        var menu = Theme.Menu();
        menu.ShowCheckMargin = true;

        for (int i = 0; i < _items.Count; i++)
        {
            int index = i;
            var item = new ToolStripMenuItem(_items[i]);
            item.Checked = index == _selectedIndex;
            item.Click += (_, _) => SelectedIndex = index;
            menu.Items.Add(item);
        }

        _open = true;
        Invalidate();
        menu.Closed += (_, _) =>
        {
            _open = false;
            Invalidate();
        };
        menu.Show(this, new Point(0, Height));
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        PaintHelper.Fill(g, this, _surface);
        g.SmoothingMode = SmoothingMode.AntiAlias;

        var rect = new Rectangle(0, 0, Width - 1, Height - 1);
        using var path = Theme.Round(rect, Ui.S(9));
        using (var fill = new SolidBrush(Enabled ? Theme.Field : Theme.Soft))
        {
            g.FillPath(fill, path);
        }

        using (var pen = new Pen(_open || _hover ? Theme.BorderFocus : Theme.Border, _open ? Ui.Sf(1.6f) : 1f))
        {
            g.DrawPath(pen, path);
        }

        TextRenderer.DrawText(
            g,
            SelectedItem,
            Font,
            new Rectangle(Ui.S(12), 0, Width - Ui.S(38), Height),
            Enabled ? Theme.Ink : Theme.Muted,
            TextFormatFlags.VerticalCenter | TextFormatFlags.NoPrefix | TextFormatFlags.EndEllipsis);

        int cx = Width - Ui.S(16);
        int cy = Height / 2;
        int w = Ui.S(4);
        int h = Ui.S(2);
        using var chevron = new Pen(_open || _hover ? Theme.Accent : Theme.Muted, Ui.Sf(1.6f))
        {
            StartCap = LineCap.Round,
            EndCap = LineCap.Round,
        };
        g.DrawLines(chevron, new[]
        {
            new PointF(cx - w, cy - h),
            new PointF(cx, cy + h),
            new PointF(cx + w, cy - h),
        });

        base.OnPaint(e);
    }
}

internal sealed class ModernMenuColors : ProfessionalColorTable
{
    public override Color MenuItemSelected => Theme.SoftHover;

    public override Color MenuItemBorder => Theme.SoftHover;

    public override Color MenuBorder => Theme.Border;

    public override Color ToolStripDropDownBackground => Theme.Card;

    public override Color ImageMarginGradientBegin => Theme.Card;

    public override Color ImageMarginGradientMiddle => Theme.Card;

    public override Color ImageMarginGradientEnd => Theme.Card;

    public override Color SeparatorDark => Theme.Border;

    public override Color SeparatorLight => Theme.Card;

    public override Color CheckBackground => Theme.Soft;

    public override Color CheckSelectedBackground => Theme.SoftHover;

    public override Color CheckPressedBackground => Theme.SoftHover;

    public override Color MenuItemSelectedGradientBegin => Theme.SoftHover;

    public override Color MenuItemSelectedGradientEnd => Theme.SoftHover;

    public override Color MenuItemPressedGradientBegin => Theme.Soft;

    public override Color MenuItemPressedGradientEnd => Theme.Soft;
}

/// <summary>Меню в цветах темы: штатная галочка тёмная и на тёмном фоне не видна.</summary>
internal sealed class ModernMenuRenderer : ToolStripProfessionalRenderer
{
    public ModernMenuRenderer()
        : base(new ModernMenuColors())
    {
    }

    protected override void OnRenderItemCheck(ToolStripItemImageRenderEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;

        var r = e.ImageRectangle;
        if (r.Width < 4 || r.Height < 4)
        {
            base.OnRenderItemCheck(e);
            return;
        }

        using var pen = new Pen(Theme.Accent, Math.Max(1.6f, r.Height * 0.10f))
        {
            StartCap = LineCap.Round,
            EndCap = LineCap.Round,
            LineJoin = LineJoin.Round,
        };
        g.DrawLines(pen, new[]
        {
            new PointF(r.Left + r.Width * 0.16f, r.Top + r.Height * 0.52f),
            new PointF(r.Left + r.Width * 0.42f, r.Top + r.Height * 0.76f),
            new PointF(r.Left + r.Width * 0.86f, r.Top + r.Height * 0.24f),
        });
    }
}

/// <summary>Мелкая подпись над полем.</summary>
internal static class Caption
{
    public static Label Make(string text, Color? surface = null) => new()
    {
        Text = text,
        AutoSize = false,
        Font = Theme.F(8.5F),
        ForeColor = Theme.Muted,
        BackColor = surface ?? Theme.Bg,
        TextAlign = ContentAlignment.MiddleLeft,
    };
}
