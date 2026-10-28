using System.Windows.Forms;

namespace SimpleAutoClicker;

/// <summary>Выбор оформления: системное, светлое или тёмное.</summary>
internal static class ThemeSwitcher
{
    public static void ShowMenu(Control anchor)
    {
        var menu = Theme.Menu();
        menu.ShowCheckMargin = true;

        foreach ((AppTheme mode, string text) in Options())
        {
            menu.Items.Add(CreateItem(anchor, mode, text));
        }

        menu.Show(anchor, new Point(0, anchor.Height));
    }

    /// <summary>Пункты «Theme» для обычного меню (Options в окне записи).</summary>
    public static void FillSubmenu(ToolStripMenuItem parent, Control anchor)
    {
        parent.ForeColor = Theme.Ink;

        var dropDown = parent.DropDown;
        dropDown.BackColor = Theme.Card;
        dropDown.ForeColor = Theme.Ink;
        dropDown.Renderer = new ModernMenuRenderer();

        foreach ((AppTheme mode, string text) in Options())
        {
            dropDown.Items.Add(CreateItem(anchor, mode, text));
        }
    }

    private static (AppTheme Mode, string Text)[] Options() => new[]
    {
        (AppTheme.System, "System"),
        (AppTheme.Light, "Light"),
        (AppTheme.Dark, "Dark"),
    };

    private static ToolStripMenuItem CreateItem(Control anchor, AppTheme mode, string text)
    {
        var item = new ToolStripMenuItem(text)
        {
            Checked = Theme.Mode == mode,
            ForeColor = Theme.Ink,
        };
        item.Click += (_, _) => Choose(anchor, mode);
        return item;
    }

    private static void Choose(Control anchor, AppTheme mode)
    {
        if (mode == Theme.Mode)
        {
            return;
        }

        // окна создаются уже в выбранных цветах, поэтому проще перезапуститься
        bool ok = Msg.Confirm(
            anchor.FindForm(),
            "The theme is applied on startup, so the app will be restarted.\n" +
            "An unsaved recording will be lost.",
            "Restart");

        if (!ok)
        {
            return;
        }

        AppSettings.Current.Theme = mode;
        AppSettings.Current.Save();
        AppControl.Restart();
    }
}
