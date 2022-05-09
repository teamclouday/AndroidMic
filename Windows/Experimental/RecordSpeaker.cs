using System;
using System.Threading;
using NAudio.Wave;

namespace Experimental
{
    partial class Program
    {
        public static void CaptureSpeaker()
        {
            var capture = new WasapiLoopbackCapture();
            Console.WriteLine("Capture Device Format: (" +
                capture.WaveFormat.SampleRate + "," +
                capture.WaveFormat.BitsPerSample + "," +
                capture.WaveFormat.Channels + ")");

            var targetWaveFormat = new WaveFormat(25000, 16, 2);
            Console.WriteLine("Target Format: (" +
                targetWaveFormat.SampleRate + "," +
                targetWaveFormat.BitsPerSample + "," +
                targetWaveFormat.Channels + ")");

            byte[] buffer = null;
            var bufferSteamer = new BufferedWaveProvider(capture.WaveFormat);
            bufferSteamer.ReadFully = true;
            IWaveProvider converter = BuildPipeline(bufferSteamer, capture.WaveFormat, targetWaveFormat, out float multiplier);

            var writer = new WaveFileWriter("recorded.wav", targetWaveFormat);

            capture.DataAvailable += (s, a) =>
            {
                if (buffer == null || buffer.Length < a.Buffer.Length)
                    buffer = new byte[a.Buffer.Length];
                bufferSteamer.AddSamples(a.Buffer, 0, a.BytesRecorded);
                int bytesToRead = (int)(a.BytesRecorded * multiplier);
                if (bytesToRead % 2 != 0) bytesToRead -= 1;
                var convertedBytes = converter.Read(buffer, 0, bytesToRead);
                Console.WriteLine("Recieved " + a.BytesRecorded + ", read " + convertedBytes);
                writer.Write(buffer, 0, convertedBytes);
            };
            capture.RecordingStopped += (s, a) =>
            {
                if (buffer == null || buffer.Length < bufferSteamer.BufferedBytes)
                    buffer = new byte[bufferSteamer.BufferedBytes];
                int bytesToRead = (int)(bufferSteamer.BufferedBytes * multiplier);
                if (bytesToRead % 2 != 0) bytesToRead -= 1;
                var convertedBytes = converter.Read(buffer, 0, bytesToRead);
                writer.Write(buffer, 0, convertedBytes);
                writer.Dispose();
                writer = null;
                capture.Dispose();
            };
            capture.StartRecording();
            // recording seconds
            int numSeconds = 10;
            while (capture.CaptureState != NAudio.CoreAudioApi.CaptureState.Stopped)
            {
                Thread.Sleep(1000);
                numSeconds -= 1;
                if (numSeconds < 0) capture.StopRecording();
            }
            Console.WriteLine("Recording stopped");
        }

        // build a conversion pipeline
        // assume output is (16000, 16, 1) format
        public static IWaveProvider BuildPipeline(IWaveProvider provider, WaveFormat input, WaveFormat output, out float multiplier)
        {
            multiplier = 1.0f;
            // resample
            if (input.SampleRate != output.SampleRate)
            {
                // resample
                provider = new MediaFoundationResampler(provider, output.SampleRate)
                {
                    ResamplerQuality = 60
                };
                multiplier *= (float)input.SampleRate / output.SampleRate;
            }
            // convert encoding
            if (input.Encoding == WaveFormatEncoding.IeeeFloat)
            {
                provider = new WaveFloatTo16Provider(provider);
                multiplier *= 2.0f;
            }
            // stereo to mono
            if (input.Channels != output.Channels)
            {
                provider = new StereoToMonoProvider16(provider);
                multiplier *= 2.0f;
            }
            multiplier = 1.0f / multiplier;
            return provider;
        }
    }
}