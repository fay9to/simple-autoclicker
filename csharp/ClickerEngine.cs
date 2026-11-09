using System.Diagnostics;

namespace SimpleAutoClicker;

/// <summary>Настройки одного запуска автокликера.</summary>
public sealed class ClickSettings
{
    public int IntervalMs { get; init; }
    public MouseButtonKind Button { get; init; }
    public bool DoubleClick { get; init; }
    public bool RepeatForever { get; init; }
    public int RepeatCount { get; init; }
    public bool FixedPosition { get; init; }
    public int X { get; init; }
    public int Y { get; init; }
}

/// <summary>Поток, который кликает с заданным интервалом.</summary>
public sealed class ClickerEngine
{
    private Thread? _thread;
    private volatile bool _running;

    public event Action<int>? Clicked;
    public event Action? Finished;

    public bool IsRunning => _running;

    public void Start(ClickSettings settings)
    {
        if (_running)
        {
            return;
        }

        _running = true;
        _thread = new Thread(() => Loop(settings))
        {
            IsBackground = true,
            Name = "auto-clicker",
            Priority = ThreadPriority.AboveNormal,
        };
        _thread.Start();
    }

    public void Stop() => _running = false;

    private void Loop(ClickSettings s)
    {
        int interval = Math.Max(1, s.IntervalMs);
        long periodTicks = Math.Max(1, Stopwatch.Frequency * (long)interval / 1000);
        long next = Stopwatch.GetTimestamp();
        int count = 0;

        try
        {
            while (_running)
            {
                if (s.FixedPosition)
                {
                    Native.SetCursorPos(s.X, s.Y);
                }

                Native.Click(s.Button, s.DoubleClick);
                count++;
                Clicked?.Invoke(count);

                if (!s.RepeatForever && count >= Math.Max(1, s.RepeatCount))
                {
                    break;
                }

                next += periodTicks;
                long now = Stopwatch.GetTimestamp();
                if (next < now)
                {
                    // не «догоняем» пропущенное время, иначе после тормозов пойдёт шквал кликов
                    next = now;
                }

                SleepUntil(next);

            }
        }
        finally
        {
            _running = false;
            Finished?.Invoke();
        }
    }

    private void SleepUntil(long targetTimestamp)
    {
        while (_running)
        {
            long remaining = targetTimestamp - Stopwatch.GetTimestamp();
            if (remaining <= 0)
            {
                return;
            }

            double ms = remaining * 1000.0 / Stopwatch.Frequency;
            if (ms > 2.0)
            {
                Thread.Sleep((int)(ms - 1));
            }
            else
            {
                Thread.SpinWait(150);
            }
        }
    }
}
