param(
    [string]$Name = "shot",
    [switch]$FullScreen
)

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class CapWin {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
    [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
}
"@
[void][CapWin]::SetProcessDPIAware()

$dir = Join-Path $PSScriptRoot "..\docs"
$path = Join-Path $dir "$Name.png"

if ($FullScreen) {
    $s = [Windows.Forms.SystemInformation]::VirtualScreen
    $bmp = New-Object Drawing.Bitmap $s.Width, $s.Height
    $g = [Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($s.Left, $s.Top, 0, 0, $bmp.Size)
} else {
    $p = Get-Process SimpleAutoClicker -ErrorAction Stop
    $h = $p.MainWindowHandle
    if ([CapWin]::IsIconic($h)) { [void][CapWin]::ShowWindow($h, 9) }
    $wshell = New-Object -ComObject WScript.Shell
    for ($i = 0; $i -lt 3 -and [CapWin]::GetForegroundWindow() -ne $h; $i++) {
        [void]$wshell.AppActivate($p.Id)
        Start-Sleep -Milliseconds 250
    }
    if ([CapWin]::GetForegroundWindow() -ne $h) {
        [CapWin]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
        [void][CapWin]::SetForegroundWindow($h)
        [CapWin]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
        [void][CapWin]::BringWindowToTop($h)
        Start-Sleep -Milliseconds 350
    }
    Start-Sleep -Milliseconds 300
    $r = New-Object CapWin+RECT
    [void][CapWin]::GetWindowRect($h, [ref]$r)
    $w = $r.Right - $r.Left
    $hh = $r.Bottom - $r.Top
    $bmp = New-Object Drawing.Bitmap $w, $hh
    $g = [Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
}

$bmp.Save($path, [Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
"saved $path"
