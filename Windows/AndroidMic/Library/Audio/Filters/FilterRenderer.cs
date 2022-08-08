using System;
using System.Windows;
using System.Windows.Media;
using System.Windows.Shapes;
using System.Windows.Controls;
using NAudio.Wave;

namespace AndroidMic.Audio
{
    class AudioPeaks
    {
        public readonly int WINDOW;
        private float valMax;
        private float valMin;
        private int counter;

        public AudioPeaks(int size)
        {
            WINDOW = size;
            valMax = 0.0f;
            valMin = 0.0f;
            counter = 0;
        }

        // update min, max in current window
        // restrict to range [-1, 1]
        public void Consume(float val)
        {
            valMax = Math.Min(Math.Max(valMax, val), 1.0f);
            valMin = Math.Max(Math.Min(valMin, val), -1.0f);
            counter++;
        }

        // get next pair if window is full
        public bool NextPair(ref float valMaxCopy, ref float valMinCopy)
        {
            if (counter > WINDOW)
            {
                valMaxCopy = valMax;
                valMinCopy = valMin;
                counter = 0;
                valMax = 0.0f;
                valMin = 0.0f;
                return true;
            }
            else return false;
        }
    }

    public class FilterRenderer : ISampleProvider
    {
        public readonly int MAX_POINT_COUNT = 100;
        public readonly int IMG_W = 340;
        public readonly int IMG_H = 100;
        private readonly float POINT_INTERVAL;
        private readonly float PEAK_MULTIPLIER = 0.8f;

        public Canvas RenderCanvas;
        private readonly Polygon streamGroup;
        private readonly Point[] audioPeaks;

        private readonly ISampleProvider source;
        private readonly AudioPeaks peaks;

        public FilterRenderer(ISampleProvider source, int speed = 5, FilterRenderer prev = null)
        {
            this.source = source;
            POINT_INTERVAL = (float)IMG_W / MAX_POINT_COUNT;
            streamGroup = new Polygon
            {
                Stroke = Brushes.DarkCyan,
                StrokeThickness = 1.5,
                Fill = Brushes.LightCyan,
                HorizontalAlignment = HorizontalAlignment.Center,
                VerticalAlignment = VerticalAlignment.Center
            };
            audioPeaks = new Point[MAX_POINT_COUNT * 2];
            for (int i = 0; i < MAX_POINT_COUNT; i++)
            {
                audioPeaks[i].X = i * POINT_INTERVAL;
                audioPeaks[i].Y = IMG_H * 0.5f;
            }
            for (int i = MAX_POINT_COUNT; i > 0; i--)
            {
                audioPeaks[MAX_POINT_COUNT * 2 - i].X = i * POINT_INTERVAL;
                audioPeaks[MAX_POINT_COUNT * 2 - i].Y = IMG_H * 0.5f;
            }
            // the larger the speed, the slower it updates
            peaks = new AudioPeaks(MAX_POINT_COUNT * speed);
            // copy prev canvas pointer
            if (prev != null)
            {
                ApplyToCanvas(prev.RenderCanvas);
            }
        }

        // add polygon to canvas
        public void ApplyToCanvas(Canvas c)
        {
            RenderCanvas = c;
            RenderCanvas.Children.Clear();
            RenderCanvas.Children.Add(streamGroup);
        }

        // read call
        public int Read(float[] buffer, int offset, int sampleCount)
        {
            float nextMax = 0.0f, nextMin = 0.0f;
            int samplesRead = source.Read(buffer, offset, sampleCount);
            for (int n = 0; n < sampleCount; n++)
            {
                peaks.Consume(PEAK_MULTIPLIER * buffer[n + offset]);
                if (peaks.NextPair(ref nextMax, ref nextMin))
                {
                    Application.Current.Dispatcher.Invoke(new Action(() =>
                    {
                        Push(nextMax, nextMin);
                    }));
                }
            }
            return samplesRead;
        }

        // refresh stream group and update UI
        private void Push(float maxVal, float minVal)
        {
            for (int i = 0; i < MAX_POINT_COUNT - 1; i++)
                audioPeaks[i].Y = audioPeaks[i + 1].Y;
            for (int i = MAX_POINT_COUNT * 2 - 1; i > MAX_POINT_COUNT; i--)
                audioPeaks[i].Y = audioPeaks[i - 1].Y;
            audioPeaks[MAX_POINT_COUNT - 1].Y = IMG_H * 0.5f * (1.0f + maxVal);
            audioPeaks[MAX_POINT_COUNT].Y = IMG_H * 0.5f * (1.0f + minVal);
            PointCollection pc = new PointCollection(audioPeaks.Length);
            foreach (var p in audioPeaks) pc.Add(p);
            streamGroup.Points = pc;
        }

        public WaveFormat WaveFormat => source.WaveFormat;
    }
}
