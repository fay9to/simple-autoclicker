param(
    [double]$X,
    [double]$Y,
    [switch]$Double
)

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class UiWin {
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
}
"@
[void][UiWin]::SetProcessDPIAware()

$p = Get-Process SimpleAutoClicker -ErrorAction Stop
$h = $p.MainWindowHandle

# AppActivate restores the foreground reliably, even after clicking the desktop.
$wshell = New-Object -ComObject WScript.Shell
for ($i = 0; $i -lt 3 -and [UiWin]::GetForegroundWindow() -ne $h; $i++) {
    [void]$wshell.AppActivate($p.Id)
    Start-Sleep -Milliseconds 250
}

if ([UiWin]::GetForegroundWindow() -ne $h) {
    # ALT trick: releases the foreground lock so SetForegroundWindow succeeds.
    [UiWin]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
    [void][UiWin]::SetForegroundWindow($h)
    [UiWin]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    [void][UiWin]::BringWindowToTop($h)
    Start-Sleep -Milliseconds 350
}

$origin = New-Object UiWin+POINT
[void][UiWin]::ClientToScreen($h, [ref]$origin)

$scale = 1.5
$sx = [int][Math]::Round($origin.X + $X * $scale)
$sy = [int][Math]::Round($origin.Y + $Y * $scale)

[void][UiWin]::SetCursorPos($sx, $sy)
Start-Sleep -Milliseconds 150

if ([UiWin]::GetForegroundWindow() -ne $h) {
    # The first click would only activate the window, so use it as a warm-up.
    [UiWin]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 60
    [UiWin]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 250
    [void][UiWin]::SetCursorPos($sx, $sy)
    Start-Sleep -Milliseconds 120
}

[UiWin]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 60
[UiWin]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
if ($Double) {
    Start-Sleep -Milliseconds 60
    [UiWin]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 60
    [UiWin]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}
"clicked design ($X,$Y) -> screen ($sx,$sy) foreground=$([UiWin]::GetForegroundWindow() -eq $h)"
