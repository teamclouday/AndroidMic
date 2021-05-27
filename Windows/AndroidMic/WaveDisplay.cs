using System;
using System.Collections.Generic;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Shapes;

namespace AndroidMic
{
    // Custom class to update canvas for displaying waveform of audio input
    public class WaveDisplay
    {
        private readonly List<Tuple<short, short>> buffer = new List<Tuple<short, short>>();
        private const int MAX_POINT_NUM = 100;
        private const int IMAGE_WIDTH = 340;
        private const int IMAGE_HEIGHT = 100;
        private const float POINT_INTERVAL = (float)IMAGE_WIDTH / MAX_POINT_NUM;
        private readonly Point[] mCurve = new Point[MAX_POINT_NUM * 2];
        private readonly Polygon mPolygon = new Polygon();
        private readonly Canvas canvas;

        public WaveDisplay(Canvas c)
        {
            canvas = c;
            // prepare polygon information
            mPolygon.Stroke = Brushes.DarkCyan;
            mPolygon.StrokeThickness = 1.5;
            mPolygon.Fill = Brushes.LightCyan;
            mPolygon.HorizontalAlignment = HorizontalAlignment.Center;
            mPolygon.VerticalAlignment = VerticalAlignment.Center;
            InitCurve();
            Refresh();
        }

        // add new data to buffer and update canvas
        public void AddData(short dataPos, short dataNeg)
        {
            buffer.Add(Tuple.Create(dataPos, dataNeg));
            while (buffer.Count > MAX_POINT_NUM) buffer.RemoveAt(0);
            Refresh();
        }

        // reset buffer and update canvas
        public void Reset()
        {
            InitCurve();
            buffer.Clear();
            Refresh();
        }

        // init list of Point as curve
        private void InitCurve()
        {
            for (int i = 0; i < MAX_POINT_NUM * 2; i++)
            {
                mCurve[i].X = i * POINT_INTERVAL;
                mCurve[i].Y = IMAGE_HEIGHT / 2.0f;
            }
        }

        // reference: https://stackoverflow.com/questions/2042155/high-quality-graph-waveform-display-component-in-c-sharp
        // reference: https://stackoverflow.com/questions/1215326/open-source-c-sharp-code-to-present-wave-form
        // refresh curve points and update canvas
        private void Refresh()
        {
            int remainingCount = MAX_POINT_NUM - buffer.Count;
            // fill curve
            for (int i = 0; i < buffer.Count; i++)
            {
                float yMax = ((float)buffer[i].Item1 - short.MinValue) / ushort.MaxValue * IMAGE_HEIGHT;
                float yMin = ((float)buffer[i].Item2 - short.MinValue) / ushort.MaxValue * IMAGE_HEIGHT;
                float xPos = (remainingCount + i + 1) * POINT_INTERVAL;
                mCurve[i + remainingCount].X = xPos;
                mCurve[i + remainingCount].Y = yMax;
                mCurve[MAX_POINT_NUM * 2 - 1 - i - remainingCount].X = xPos;
                mCurve[MAX_POINT_NUM * 2 - 1 - i - remainingCount].Y = yMin;
            }
            PointCollection collection = new PointCollection();
            for (int i = 0; i < mCurve.Length; i++) collection.Add(mCurve[i]);
            mPolygon.Points = collection;
            // update canvas
            canvas.Children.Clear();
            canvas.Children.Add(mPolygon);
            //
            // // The following implementation has worse performance because MemoryStream has to be GC every update
            //
            // Color cArea = Color.FromRgb(214, 161, 255);
            // Color cBorder = Color.FromRgb(105, 0, 186);
            //    using (Graphics g = Graphics.FromImage(mImage))
            //    {
            //        g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
            //        g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
            //        g.Clear(Color.Black);
            //        Pen penBorder = new Pen(cBorder);
            //        penBorder.LineJoin = System.Drawing.Drawing2D.LineJoin.Round;
            //        penBorder.Width = 2.0f;
            //        Brush brushArea = new SolidBrush(cArea);
            //        int remainingCount = MAX_POINT_NUM - buffer.Count;
            //        // fill curve
            //        for(int i = 0; i < buffer.Count; i++)
            //        {
            //            float yMax = IMAGE_HEIGHT - ((buffer[i].Item1 - short.MinValue) / ushort.MaxValue * IMAGE_HEIGHT);
            //            float yMin = IMAGE_HEIGHT - ((buffer[i].Item2 - short.MinValue) / ushort.MaxValue * IMAGE_HEIGHT);
            //            float xPos = (remainingCount + i) * POINT_INTERVAL;
            //            mCurve[i + remainingCount].X = xPos;
            //            mCurve[i + remainingCount].Y = yMax;
            //            mCurve[MAX_POINT_NUM * 2 - 1 - i - remainingCount].X = xPos;
            //            mCurve[MAX_POINT_NUM * 2 - 1 - i - remainingCount].Y = yMin;
            //        }
            //        g.FillClosedCurve(brushArea, mCurve, System.Drawing.Drawing2D.FillMode.Winding, 0.15f);
            //        g.DrawClosedCurve(penBorder, mCurve, 0.15f, System.Drawing.Drawing2D.FillMode.Winding);
            //        penBorder.Dispose();
            //        brushArea.Dispose();
            //    }
            //    MemoryStream stream = new MemoryStream();
            //    mImage.Save(stream, System.Drawing.Imaging.ImageFormat.Bmp);
            //    stream.Position = 0;
            //    Image = new BitmapImage();
            //    Image.BeginInit();
            //    Image.StreamSource = stream;
            //    Image.EndInit();
        }
    }
}
