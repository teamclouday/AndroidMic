using System;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;
using librnnoise;
using NAudio.Utils;

namespace AndroidMic.Audio
{
    // use rnnoise preprocessor
    // reference: https://github.com/xiph/rnnoise/blob/master/examples/rnnoise_demo.c
    public class FilterRnnoise : ISampleProvider, IDisposable
    {
        private readonly int FRAME_SIZE = 480;
        private readonly float[] processBuffer;

        private readonly float SCALE_INPUT = short.MaxValue;
        private readonly float SCALE_OUTPUT = 1.0f / short.MaxValue;

        private readonly ISampleProvider source;

        private readonly IntPtr denoiseState;

        public FilterRnnoise(ISampleProvider source, FilterRnnoise _ = null)
        {
            this.source = new WdlResamplingSampleProvider(source, 48000);

            // allocate process buffer
            processBuffer = new float[FRAME_SIZE];

            // initialize denoise state
            denoiseState = Rnnoise.rnnoise_create();
        }

        public int Read(float[] buffer, int offset, int sampleCount)
        {
            int samplesRead = source.Read(buffer, offset, sampleCount);
            ApplyDenoise(buffer, offset, samplesRead);
            return samplesRead;
        }

        public void Dispose()
        {
            Rnnoise.rnnoise_destroy(denoiseState);
        }

        private void ApplyDenoise(float[] buffer, int offset, int samplesRead)
        {
            if (samplesRead <= 0) return;

            int toRead = samplesRead;
            while (toRead > 0)
            {
                // copy buffer
                int nextRead = Math.Min(toRead, FRAME_SIZE);
                //Buffer.BlockCopy(buffer, offset, processBuffer, 0, nextRead);
                for (var idx = 0; idx < nextRead; ++idx)
                {
                    processBuffer[idx] = buffer[offset + idx] * SCALE_INPUT;
                }
                if (nextRead < FRAME_SIZE) Array.Clear(processBuffer, nextRead, FRAME_SIZE - nextRead);

                // process audio
                Rnnoise.rnnoise_process_frame(denoiseState, processBuffer, processBuffer);

                // copy back
                //Buffer.BlockCopy(processBuffer, 0, buffer, offset, nextRead);
                for (var idx = 0; idx < nextRead; ++idx)
                {
                    buffer[offset + idx] = Math.Max(-1.0f, Math.Min(1.0f, processBuffer[idx] * SCALE_OUTPUT));
                }

                toRead -= nextRead;
                offset += nextRead;
            }
        }

        public WaveFormat WaveFormat => source.WaveFormat;
    }
}
