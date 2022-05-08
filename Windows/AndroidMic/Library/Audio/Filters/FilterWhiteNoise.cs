using System;
using NAudio.Wave;

namespace AndroidMic.Audio
{
    // add white noise to audio stream

    public class FilterWhiteNoise : ISampleProvider
    {
        public enum ConfigTypes
        {
            ConfigStrength = 0
        }

        private readonly Random rnd;

        private readonly ISampleProvider source;

        public FilterWhiteNoise(ISampleProvider source, FilterWhiteNoise prev = null)
        {
            this.source = source;
            rnd = new Random();
            Strength = 0.0f;
            if(prev != null)
            {
                Strength = prev.Strength;
            }
        }

        // read call
        public int Read(float[] buffer, int offset, int sampleCount)
        {
            int samplesRead = source.Read(buffer, offset, sampleCount);
            ApplyNoise(buffer, offset, samplesRead);
            return samplesRead;
        }

        private void ApplyNoise(float[] buffer, int offset, int samplesRead)
        {
            if (samplesRead <= 0 || Strength == 0.0f) return;
            for(int i = 0; i < samplesRead; i++)
            {
                // add noise such that range is still between -1 and 1
                //buffer[i + offset] = buffer[i + offset] * 0.5f * (1.0f - Strength) + ((float)rnd.NextDouble() - 0.5f) * Strength;
                buffer[i + offset] = buffer[i + offset] * (1.0f - Strength) + ((float)TruncatedNormal(Strength * 0.5) - 0.5f) * Strength;
            }
        }

        // get random normal distribution
        // reference: https://stackoverflow.com/questions/218060/random-gaussian-variables
        private double TruncatedNormal(double std, double mean = 0.0, double vMin = -1.0, double vMax = 1.0)
        {
            double u1 = 1.0 - rnd.NextDouble();
            double u2 = 1.0 - rnd.NextDouble();
            double randStdNormal = Math.Sqrt(-2.0 * Math.Log(u1)) *
                         Math.Sin(2.0 * Math.PI * u2); //random normal(0,1)
            return Math.Max(vMin, Math.Min(vMax, mean + std * randStdNormal));
        }

        public WaveFormat WaveFormat => source.WaveFormat;

        // strength: [-1.0, 1.0]
        public float Strength { get; set; }
    }
}
