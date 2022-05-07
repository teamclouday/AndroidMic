using System;
using System.IO;
using System.Diagnostics;
using Microsoft.Win32;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;

namespace AndroidMic.Audio
{
    class FilterRepeatTrack : ISampleProvider, IDisposable
    {
        public enum ConfigTypes
        {
            ConfigStrength = 0,
            ConfigRepeat = 1,
            ConfigSelectFile = 2
        }

        private readonly ISampleProvider source;

        private volatile AudioFileReader reader;
        private volatile WdlResamplingSampleProvider resampler;
        private float[] readerBuffer;

        public FilterRepeatTrack(ISampleProvider source, FilterRepeatTrack prev = null)
        {
            this.source = source;
            Strength = 0.0f;
            Repeat = false;
            if (prev != null)
            {
                Strength = prev.Strength;
                Repeat = prev.Repeat;
                prev.Dispose();
            }
        }

        public string SelectFile()
        {
            OpenFileDialog dialog = new OpenFileDialog
            {
                Filter = "MP3 File (*.mp3)|*.mp3|WAV File (*.wav)|*.wav|All Files (*.*)|*.*"
            };
            if(dialog.ShowDialog() == true)
            {
                return LoadFile(dialog.FileName);
            }
            return "";
        }

        private string LoadFile(string filepath)
        {
            try
            {
                reader = new AudioFileReader(filepath);
                resampler = new WdlResamplingSampleProvider(reader.ToSampleProvider().ToMono(), WaveFormat.SampleRate);
            } catch(Exception e)
            {
                Debug.WriteLine("[FilterRepeatTrack]: " + e.Message);
                reader = null;
                resampler = null;
                return "";
            }
            return Path.GetFileName(filepath);
        }

        // read call
        public int Read(float[] buffer, int offset, int sampleCount)
        {
            int samplesRead = source.Read(buffer, offset, sampleCount);
            if (readerBuffer == null || readerBuffer.Length < buffer.Length)
                readerBuffer = new float[buffer.Length];
            if (reader != null && Strength != 0.0f)
            {
                int resampledOffset = offset;
                int resampledRead = resampler.Read(readerBuffer, offset, samplesRead);
                // reset position if no more reads
                while (resampledRead < samplesRead && Repeat)
                {
                    reader.Position = 0;
                    int resampledReadNew = resampler.Read(readerBuffer, resampledOffset, samplesRead - resampledRead);
                    // if no reads after reset position, this stream has no data
                    if (resampledReadNew <= 0) break;
                    resampledRead += resampledReadNew;
                    resampledOffset += resampledReadNew;
                }
                for(int i = 0; i < samplesRead; i++)
                {
                    buffer[i + offset] = buffer[i + offset] * (1.0f - Strength) +
                        ReadFromBuffer(i, offset, resampledRead) * Strength;
                }
            }
            return samplesRead;
        }

        private float ReadFromBuffer(int index, int offset, int samplesRead)
        {
            if ((index + offset) >= readerBuffer.Length || index >= samplesRead) return 0.0f;
            return readerBuffer[index + offset];
        }

        // dispose audio stream
        public void Dispose()
        {
            reader?.Dispose();
        }

        public WaveFormat WaveFormat => source.WaveFormat;

        // strength: [0.0, 1.0]
        public float Strength { get; set; }
        
        // whether to repeat
        public bool Repeat { get; set; }
    }
}
