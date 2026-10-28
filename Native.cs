using System.Runtime.InteropServices;

namespace SimpleAutoClicker;

/// <summary>Тонкая обёртка над Win32: глобальные горячие клавиши, эмуляция мыши, низкоуровневый хук.</summary>
internal static class Native
{
    public const int WM_HOTKEY = 0x0312;
    public const int WH_MOUSE_LL = 14;

    public const uint INPUT_MOUSE = 0;
    public const uint MOUSEEVENTF_MOVE = 0x0001;
    public const uint MOUSEEVENTF_ABSOLUTE = 0x8000;
    public const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    public const uint MOUSEEVENTF_LEFTUP = 0x0004;
    public const uint MOUSEEVENTF_RIGHTDOWN = 0x0008;
    public const uint MOUSEEVENTF_RIGHTUP = 0x0010;
    public const uint MOUSEEVENTF_MIDDLEDOWN = 0x0020;
    public const uint MOUSEEVENTF_MIDDLEUP = 0x0040;

    public const int WM_MOUSEMOVE = 0x0200;
    public const int WM_LBUTTONDOWN = 0x0201;
    public const int WM_LBUTTONUP = 0x0202;
    public const int WM_RBUTTONDOWN = 0x0204;
    public const int WM_RBUTTONUP = 0x0205;
    public const int WM_MBUTTONDOWN = 0x0207;
    public const int WM_MBUTTONUP = 0x0208;

    [StructLayout(LayoutKind.Sequential)]
    public struct INPUT
    {
        public uint type;
        public InputUnion u;
    }

    [StructLayout(LayoutKind.Explicit)]
    public struct InputUnion
    {
        [FieldOffset(0)] public MOUSEINPUT mi;
        [FieldOffset(0)] public KEYBDINPUT ki;
        [FieldOffset(0)] public HARDWAREINPUT hi;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct MOUSEINPUT
    {
        public int dx;
        public int dy;
        public uint mouseData;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct KEYBDINPUT
    {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct HARDWAREINPUT
    {
        public uint uMsg;
        public ushort wParamL;
        public ushort wParamH;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT
    {
        public int X;
        public int Y;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct MSLLHOOKSTRUCT
    {
        public POINT pt;
        public uint mouseData;
        public uint flags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    public delegate IntPtr LowLevelMouseProc(int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool UnregisterHotKey(IntPtr hWnd, int id);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int x, int y);

    [DllImport("user32.dll")]
    public static extern bool GetCursorPos(out POINT p);

    [DllImport("user32.dll")]
    public static extern int GetSystemMetrics(int index);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr SetWindowsHookEx(int idHook, LowLevelMouseProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll")]
    public static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr GetModuleHandle(string? lpModuleName);

    [DllImport("dwmapi.dll")]
    private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attribute, ref int value, int size);

    /// <summary>Красит заголовок окна в тёмный цвет (Windows 10 1809+ и Windows 11).</summary>
    public static void ApplyDarkTitleBar(IntPtr hwnd, bool dark)
    {
        int value = dark ? 1 : 0;

        // 20 — DWMWA_USE_IMMERSIVE_DARK_MODE в актуальных сборках, 19 — в ранних
        if (DwmSetWindowAttribute(hwnd, 20, ref value, sizeof(int)) != 0)
        {
            DwmSetWindowAttribute(hwnd, 19, ref value, sizeof(int));
        }
    }

    /// <summary>Отправляет одно нажатие (одиночное или двойное) выбранной кнопкой.</summary>
    public static void Click(MouseButtonKind button, bool doubleClick)
    {
        (uint down, uint up) = button switch
        {
            MouseButtonKind.Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButtonKind.Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        };

        int pairs = doubleClick ? 2 : 1;
        var inputs = new INPUT[pairs * 2];
        for (int i = 0; i < pairs; i++)
        {
            inputs[i * 2] = MouseInput(0, 0, down);
            inputs[i * 2 + 1] = MouseInput(0, 0, up);
        }

        SendInput((uint)inputs.Length, inputs, Marshal.SizeOf<INPUT>());
    }

    /// <summary>Перемещает курсор в абсолютную точку экрана и нажимает кнопку.</summary>
    public static void ClickAt(int x, int y, MouseButtonKind button, bool doubleClick)
    {
        SetCursorPos(x, y);
        Click(button, doubleClick);
    }

    public static void MoveAbsolute(int x, int y)
    {
        int w = Math.Max(1, GetSystemMetrics(0));
        int h = Math.Max(1, GetSystemMetrics(1));
        int nx = (int)Math.Round(x * 65535.0 / (w - 1));
        int ny = (int)Math.Round(y * 65535.0 / (h - 1));
        var input = MouseInput(nx, ny, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE);
        SendInput(1, new[] { input }, Marshal.SizeOf<INPUT>());
    }

    public static void SendButton(MouseButtonKind button, bool down)
    {
        (uint d, uint u) = button switch
        {
            MouseButtonKind.Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButtonKind.Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        };
        var input = MouseInput(0, 0, down ? d : u);
        SendInput(1, new[] { input }, Marshal.SizeOf<INPUT>());
    }

    private static INPUT MouseInput(int dx, int dy, uint flags) => new()
    {
        type = INPUT_MOUSE,
        u = new InputUnion
        {
            mi = new MOUSEINPUT
            {
                dx = dx,
                dy = dy,
                mouseData = 0,
                dwFlags = flags,
                time = 0,
                dwExtraInfo = IntPtr.Zero,
            },
        },
    };
}

public enum MouseButtonKind
{
    Left,
    Right,
    Middle,
}
