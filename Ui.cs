using System.Runtime.InteropServices;

namespace SimpleAutoClicker;

/// <summary>
/// Ручное масштабирование интерфейса под DPI.
/// Всё окно описано в «дизайнерских» пикселях 96 DPI, а перед показом размеры
/// элементов умножаются на текущий коэффициент. Шрифт при этом не трогаем —
/// DPI-осведомлённое приложение само рисует его крупнее.
/// </summary>
internal static class Ui
{
    public static float Scale { get; set; } = 1f;

    public static int S(int value) => (int)Math.Round(value * Scale);

    public static float Sf(float value) => value * Scale;

    public static Size Sz(int w, int h) => new(S(w), S(h));

    public static void Apply(Form form)
    {
        form.AutoScaleMode = AutoScaleMode.None;

        if (Math.Abs(Scale - 1f) < 0.01f)
        {
            return;
        }

        form.SuspendLayout();
        var client = form.ClientSize;
        ScaleChildren(form);
        form.ClientSize = Sz(client.Width, client.Height);
        form.ResumeLayout(true);
    }

    private static void ScaleChildren(Control parent)
    {
        foreach (Control child in parent.Controls)
        {
            child.SuspendLayout();
            child.Bounds = new Rectangle(S(child.Left), S(child.Top), S(child.Width), S(child.Height));
            ScaleChildren(child);
            child.ResumeLayout(false);
        }
    }
}

internal static class Dpi
{
    [DllImport("user32.dll")]
    private static extern uint GetDpiForSystem();

    public static float Factor()
    {
        try
        {
            uint dpi = GetDpiForSystem();
            if (dpi >= 72 && dpi <= 480)
            {
                return dpi / 96f;
            }
        }
        catch (EntryPointNotFoundException)
        {
            // очень старые Windows
        }

        return 1f;
    }
}
