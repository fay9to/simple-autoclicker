using System.Windows.Forms;

namespace SimpleAutoClicker;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        Application.SetHighDpiMode(HighDpiMode.SystemAware);
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);

        AppSettings.Load();
        Theme.Apply(AppSettings.Current.Theme);

        Ui.Scale = Dpi.Factor();
        Application.Run(new MainForm());
    }
}
