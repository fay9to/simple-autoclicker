using System.Drawing.Drawing2D;
using Microsoft.Win32;

namespace SimpleAutoClicker;

internal enum IconKind
{
    None,
    Play,
    Record,
    Stop,
    More,
    Pick,
    Sun,
    Moon,
}

/// <summary>Режим оформления: как в системе, всегда светлый или всегда тёмный.</summary>
internal enum AppTheme
{
    System,
    Light,
    Dark,
}

/// <summary>
/// Палитра, шрифты и мелкие графические помощники.
/// Тема выбирается один раз при старте: цвета — статические свойства,
/// поэтому все контролы берут актуальные значения при отрисовке.
/// </summary>
internal static class Theme
{
    public static AppTheme Mode { get; private set; } = AppTheme.System;

    public static bool IsDark { get; private set; }

    // ---- поверхность ----
    public static Color Bg { get; private set; } = Color.FromArgb(245, 247, 251);

    public static Color Card { get; private set; } = Color.White;

    /// <summary>Внутренность полей ввода, кружков и чекбоксов.</summary>
    public static Color Field { get; private set; } = Color.White;

    public static Color HoverCard { get; private set; } = Color.FromArgb(250, 251, 254);

    // ---- текст ----
    public static Color Ink { get; private set; } = Color.FromArgb(30, 35, 48);

    public static Color Muted { get; private set; } = Color.FromArgb(110, 118, 136);

    public static Color DisabledInk { get; private set; } = Color.FromArgb(170, 176, 190);

    // ---- рамки и акцент ----
    public static Color Border { get; private set; } = Color.FromArgb(225, 230, 240);

    public static Color ControlBorder { get; private set; } = Color.FromArgb(180, 187, 200);

    public static Color BorderFocus { get; private set; } = Color.FromArgb(79, 107, 255);

    public static Color Accent { get; private set; } = Color.FromArgb(79, 107, 255);

    public static Color AccentHover { get; private set; } = Color.FromArgb(101, 126, 255);

    public static Color AccentDown { get; private set; } = Color.FromArgb(61, 86, 226);

    // ---- нейтральные заливки ----
    public static Color Soft { get; private set; } = Color.FromArgb(242, 244, 250);

    public static Color SoftHover { get; private set; } = Color.FromArgb(233, 237, 246);

    public static Color DisabledSurface { get; private set; } = Color.FromArgb(196, 206, 244);

    // ---- статусы ----
    public static Color Danger { get; private set; } = Color.FromArgb(226, 68, 75);

    public static Color DangerSoft { get; private set; } = Color.FromArgb(253, 236, 237);

    public static Color DangerHover { get; private set; } = Color.FromArgb(253, 240, 241);

    public static Color DangerDown { get; private set; } = Color.FromArgb(248, 219, 221);

    public static Color Green { get; private set; } = Color.FromArgb(79, 168, 96);

    /// <summary>Цвет текста и иконок на акцентной заливке.</summary>
    public static Color OnAccent { get; private set; } = Color.White;

    private static readonly string Family = PickFamily();

    // ------------------------------------------------------------------ theme

    public static void Apply(AppTheme mode)
    {
        Mode = mode;
        IsDark = mode switch
        {
            AppTheme.Dark => true,
            AppTheme.Light => false,
            _ => SystemPrefersDark(),
        };

        if (IsDark)
        {
            LoadDark();
        }
        else
        {
            LoadLight();
        }
    }

    private static void LoadLight()
    {
        Bg = Color.FromArgb(245, 247, 251);
        Card = Color.White;
        Field = Color.White;
        HoverCard = Color.FromArgb(250, 251, 254);
        Ink = Color.FromArgb(30, 35, 48);
        Muted = Color.FromArgb(110, 118, 136);
        DisabledInk = Color.FromArgb(170, 176, 190);
        Border = Color.FromArgb(225, 230, 240);
        ControlBorder = Color.FromArgb(180, 187, 200);
        BorderFocus = Color.FromArgb(79, 107, 255);
        Accent = Color.FromArgb(79, 107, 255);
        AccentHover = Color.FromArgb(101, 126, 255);
        AccentDown = Color.FromArgb(61, 86, 226);
        Soft = Color.FromArgb(242, 244, 250);
        SoftHover = Color.FromArgb(233, 237, 246);
        DisabledSurface = Color.FromArgb(196, 206, 244);
        Danger = Color.FromArgb(226, 68, 75);
        DangerSoft = Color.FromArgb(253, 236, 237);
        DangerHover = Color.FromArgb(253, 240, 241);
        DangerDown = Color.FromArgb(248, 219, 221);
        Green = Color.FromArgb(79, 168, 96);
        OnAccent = Color.White;
    }

    private static void LoadDark()
    {
        Bg = Color.FromArgb(22, 25, 32);
        Card = Color.FromArgb(30, 34, 43);
        Field = Color.FromArgb(24, 28, 36);
        HoverCard = Color.FromArgb(38, 43, 54);
        Ink = Color.FromArgb(231, 235, 243);
        Muted = Color.FromArgb(150, 159, 176);
        DisabledInk = Color.FromArgb(108, 116, 132);
        Border = Color.FromArgb(48, 54, 67);
        ControlBorder = Color.FromArgb(74, 82, 99);
        BorderFocus = Color.FromArgb(124, 140, 255);
        Accent = Color.FromArgb(108, 124, 255);
        AccentHover = Color.FromArgb(127, 142, 255);
        AccentDown = Color.FromArgb(90, 105, 230);
        Soft = Color.FromArgb(36, 41, 51);
        SoftHover = Color.FromArgb(46, 52, 64);
        DisabledSurface = Color.FromArgb(56, 63, 104);
        Danger = Color.FromArgb(255, 107, 112);
        DangerSoft = Color.FromArgb(58, 32, 38);
        DangerHover = Color.FromArgb(72, 39, 46);
        DangerDown = Color.FromArgb(84, 44, 52);
        Green = Color.FromArgb(78, 208, 125);
        OnAccent = Color.White;
    }

    /// <summary>Читает системную настройку «тёмное оформление приложений».</summary>
    private static bool SystemPrefersDark()
    {
        try
        {
            object? value = Registry.GetValue(
                @"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "AppsUseLightTheme",
                null);

            return value is int i && i == 0;
        }
        catch (Exception)
        {
            return false;
        }
    }

    /// <summary>Готовое меню в цветах темы (штатные меню всегда светлые).</summary>
    public static ContextMenuStrip Menu()
    {
        return new ContextMenuStrip
        {
            Font = F(9.5F),
            ShowImageMargin = false,
            BackColor = Card,
            ForeColor = Ink,
            Renderer = new ModernMenuRenderer(),
        };
    }

    /// <summary>Тёмный заголовок окна, если тема тёмная.</summary>
    public static void StyleWindow(Form form)
    {
        form.HandleCreated += (_, _) => Native.ApplyDarkTitleBar(form.Handle, IsDark);
        if (form.IsHandleCreated)
        {
            Native.ApplyDarkTitleBar(form.Handle, IsDark);
        }
    }

    // ------------------------------------------------------------- typography

    public static Font F(float size, FontStyle style = FontStyle.Regular) => new(Family, size, style);

    private static string PickFamily()
    {
        foreach (string name in new[] { "Segoe UI Variable Text", "Segoe UI" })
        {
            try
            {
                using var family = new FontFamily(name);
                return name;
            }
            catch (ArgumentException)
            {
                // шрифта нет — пробуем следующий
            }
        }

        return FontFamily.GenericSansSerif.Name;
    }

    // ----------------------------------------------------------------- shapes

    public static GraphicsPath Round(RectangleF r, float radius)
    {
        var path = new GraphicsPath();
        float d = radius * 2f;
        if (d <= 1f || r.Width <= d || r.Height <= d)
        {
            path.AddRectangle(r);
            return path;
        }

        path.AddArc(r.X, r.Y, d, d, 180f, 90f);
        path.AddArc(r.Right - d, r.Y, d, d, 270f, 90f);
        path.AddArc(r.Right - d, r.Bottom - d, d, d, 0f, 90f);
        path.AddArc(r.X, r.Bottom - d, d, d, 90f, 90f);
        path.CloseFigure();
        return path;
    }

    public static void DrawIcon(Graphics g, Rectangle r, IconKind kind, Color color)
    {
        g.SmoothingMode = SmoothingMode.AntiAlias;
        using var brush = new SolidBrush(color);
        switch (kind)
        {
            case IconKind.Play:
            {
                float w = r.Width;
                float h = r.Height;
                var points = new[]
                {
                    new PointF(r.Left + w * 0.16f, r.Top + h * 0.06f),
                    new PointF(r.Left + w * 0.94f, r.Top + h * 0.50f),
                    new PointF(r.Left + w * 0.16f, r.Top + h * 0.94f),
                };
                g.FillPolygon(brush, points);
                break;
            }

            case IconKind.Record:
                g.FillEllipse(brush, r);
                break;

            case IconKind.Stop:
            {
                float s = r.Width * 0.62f;
                float x = r.Left + (r.Width - s) / 2f;
                float y = r.Top + (r.Height - s) / 2f;
                using var path = Round(new RectangleF(x, y, s, s), Math.Max(1f, s * 0.16f));
                g.FillPath(brush, path);
                break;
            }

            case IconKind.More:
            {
                float d = r.Width * 0.22f;
                float y = r.Top + (r.Height - d) / 2f;
                float gap = d * 0.55f;
                float total = d * 3 + gap * 2;
                float x = r.Left + (r.Width - total) / 2f;
                for (int i = 0; i < 3; i++)
                {
                    g.FillEllipse(brush, x, y, d, d);
                    x += d + gap;
                }

                break;
            }

            case IconKind.Pick:
            {
                using var pen = new Pen(color, Math.Max(1.5f, r.Width * 0.16f));
                float cx = r.Left + r.Width / 2f;
                float cy = r.Top + r.Height / 2f;
                float rr = r.Width * 0.30f;
                g.DrawEllipse(pen, cx - rr, cy - rr, rr * 2, rr * 2);
                g.DrawLine(pen, cx, r.Top, cx, r.Top + r.Height * 0.18f);
                g.DrawLine(pen, cx, r.Bottom - r.Height * 0.18f, cx, r.Bottom);
                g.DrawLine(pen, r.Left, cy, r.Left + r.Width * 0.18f, cy);
                g.DrawLine(pen, r.Right - r.Width * 0.18f, cy, r.Right, cy);
                break;
            }

            case IconKind.Sun:
            {
                float size = Math.Min(r.Width, r.Height);
                float cx = r.Left + r.Width / 2f;
                float cy = r.Top + r.Height / 2f;
                float core = size * 0.24f;
                g.FillEllipse(brush, cx - core, cy - core, core * 2, core * 2);

                using var pen = new Pen(color, Math.Max(1.4f, size * 0.09f))
                {
                    StartCap = LineCap.Round,
                    EndCap = LineCap.Round,
                };
                for (int i = 0; i < 8; i++)
                {
                    double a = i * Math.PI / 4;
                    float r0 = core * 1.75f;
                    float r1 = size * 0.47f;
                    g.DrawLine(
                        pen,
                        cx + (float)(Math.Cos(a) * r0),
                        cy + (float)(Math.Sin(a) * r0),
                        cx + (float)(Math.Cos(a) * r1),
                        cy + (float)(Math.Sin(a) * r1));
                }

                break;
            }

            case IconKind.Moon:
            {
                float size = Math.Min(r.Width, r.Height);
                var outer = new RectangleF(
                    r.Left + (r.Width - size) / 2f,
                    r.Top + (r.Height - size) / 2f,
                    size,
                    size);

                using var full = new GraphicsPath();
                full.AddEllipse(outer);
                using var hole = new GraphicsPath();
                float d2 = size * 0.86f;
                hole.AddEllipse(outer.X + size * 0.34f, outer.Y - size * 0.10f, d2, d2);
                using var region = new Region(full);
                region.Exclude(hole);
                g.FillRegion(brush, region);
                break;
            }
        }
    }
}
