using System;
using NAudio.Wave;

namespace AndroidMic.Audio
{
    // reference: http://blogs.zynaptiq.com/bernsee/pitch-shifting-using-the-ft/
    // reference: https://github.com/filoe/cscore/blob/master/CSCore/DSP/PitchShifterInternal.cs

    public class FilterPitchShifter : ISampleProvider
    {
        public enum ConfigTypes
        {
            ConfigPitch = 0
        }

        private readonly int MAX_FRAME_LENGTH = 8192;
        private readonly int FFT_FRAME_SIZE = 2048;
        private readonly int STFT_OVERSAMPLING = 32;

        private readonly float[] gInFIFO;
        private readonly float[] gOutFIFO;
        private readonly float[] gFFTworksp;
        private readonly float[] gLastPhase;
        private readonly float[] gSumPhase;
        private readonly float[] gOutputAccum;
        private readonly float[] gAnaFreq;
        private readonly float[] gAnaMagn;
        private readonly float[] gSynFreq;
        private readonly float[] gSynMagn;
        private long gRover;

        private readonly ISampleProvider source;

        public FilterPitchShifter(ISampleProvider source, FilterPitchShifter prev = null)
        {
            this.source = source;
            gInFIFO = new float[MAX_FRAME_LENGTH];
            gOutFIFO = new float[MAX_FRAME_LENGTH];
            gFFTworksp = new float[2 * MAX_FRAME_LENGTH];
            gLastPhase = new float[MAX_FRAME_LENGTH / 2 + 1];
            gSumPhase = new float[MAX_FRAME_LENGTH / 2 + 1];
            gOutputAccum = new float[2 * MAX_FRAME_LENGTH];
            gAnaFreq = new float[MAX_FRAME_LENGTH];
            gAnaMagn = new float[MAX_FRAME_LENGTH];
            gSynFreq = new float[MAX_FRAME_LENGTH];
            gSynMagn = new float[MAX_FRAME_LENGTH];
            gRover = 0;
            // copy configs from prev filter
            if(prev != null)
            {
                PitchShift = prev.PitchShift;
            }
        }

        // read call
        public int Read(float[] buffer, int offset, int sampleCount)
        {
            int samplesRead = source.Read(buffer, offset, sampleCount);
            if (PitchShift != 1.0f)
            {
                smbPitchShift(
                    PitchShift, samplesRead, offset,
                    FFT_FRAME_SIZE, STFT_OVERSAMPLING,
                    source.WaveFormat.SampleRate, buffer);
            }
            return samplesRead;
        }

        public WaveFormat WaveFormat => source.WaveFormat;

        // The routine takes a pitchShift factor value which is between 0.5 (one octave down)
        // and 2. (one octave up). A value of exactly 1 does not change the pitch
        private float _pitchShift = 1.0f;
        public float PitchShift
        {
            get => _pitchShift;
            set => _pitchShift = Math.Min(Math.Max(0.5f, value), 2.0f);
        }

        // following functions are transformed from Stephan M. Bernsee's blog
        // URL: http://blogs.zynaptiq.com/bernsee
        // fftFrameSize defines the FFT frame size used for the processing.
        // Typical values are 1024, 2048 and 4096. It may be any value <= MAX_FRAME_LENGTH
        // but it MUST be a power of 2.
        // osamp is the STFT oversampling factor which also determines the overlap
        // between adjacent STFT frames.It should at least be 4 for moderate scaling
        // ratios. A value of 32 is recommended for best quality.
        // sampleRate takes the sample rate for the signal in unit Hz, ie. 44100 for 44.1 kHz audio.
        private void smbPitchShift(
            float pitchShift, long numSampsToProcess, long offset,
            long fftFrameSize, long osamp, long sampleRate, float[] rawData)
        {
            /* set up some handy variables */
            long fftFrameSize2 = fftFrameSize / 2;
            long stepSize = fftFrameSize / osamp;
            double freqPerBin = sampleRate / (double)fftFrameSize;
            double expct = 2.0 * Math.PI * stepSize / fftFrameSize;
            long inFifoLatency = fftFrameSize - stepSize;
            if (gRover == 0) gRover = inFifoLatency;
            /* main processing loop */
            long i, k;
            double real, imag, magn, phase, tmp, window;
            for (i = 0; i < numSampsToProcess; i++)
            {
                /* As long as we have not yet collected enough data just read in */
                gInFIFO[gRover] = rawData[i + offset];
                rawData[i + offset] = gOutFIFO[gRover - inFifoLatency];
                gRover++;

                /* now we have enough data for processing */
                if (gRover >= fftFrameSize)
                {
                    gRover = inFifoLatency;

                    /* do windowing and re,im interleave */
                    for (k = 0; k < fftFrameSize; k++)
                    {
                        window = -0.5 * Math.Cos(2.0 * Math.PI * k / fftFrameSize) + 0.5;
                        gFFTworksp[2 * k] = gInFIFO[k] * (float)window;
                        gFFTworksp[2 * k + 1] = 0.0f;
                    }


                    /* ***************** ANALYSIS ******************* */
                    /* do transform */
                    smbFft(gFFTworksp, fftFrameSize, -1);

                    /* this is the analysis step */
                    for (k = 0; k <= fftFrameSize2; k++)
                    {

                        /* de-interlace FFT buffer */
                        real = gFFTworksp[2 * k];
                        imag = gFFTworksp[2 * k + 1];

                        /* compute magnitude and phase */
                        magn = 2.0 * Math.Sqrt(real * real + imag * imag);
                        phase = Math.Atan2(imag, real);

                        /* compute phase difference */
                        tmp = phase - gLastPhase[k];
                        gLastPhase[k] = (float)phase;

                        /* subtract expected phase difference */
                        tmp -= k * expct;

                        /* map delta phase into +/- Pi interval */
                        long qpd = (long)(tmp / Math.PI);
                        if (qpd >= 0) qpd += qpd & 1;
                        else qpd -= qpd & 1;
                        tmp -= Math.PI * qpd;

                        /* get deviation from bin frequency from the +/- Pi interval */
                        tmp = osamp * tmp / (2.0 * Math.PI);

                        /* compute the k-th partials' true frequency */
                        tmp = k * freqPerBin + tmp * freqPerBin;

                        /* store magnitude and true frequency in analysis arrays */
                        gAnaMagn[k] = (float)magn;
                        gAnaFreq[k] = (float)tmp;

                    }

                    /* ***************** PROCESSING ******************* */
                    /* this does the actual pitch shifting */
                    Array.Clear(gSynMagn, 0, (int)fftFrameSize);
                    Array.Clear(gSynFreq, 0, (int)fftFrameSize);
                    for (k = 0; k <= fftFrameSize2; k++)
                    {
                        long index = (long)(k * pitchShift);
                        if (index <= fftFrameSize2)
                        {
                            gSynMagn[index] += gAnaMagn[k];
                            gSynFreq[index] = gAnaFreq[k] * pitchShift;
                        }
                    }

                    /* ***************** SYNTHESIS ******************* */
                    /* this is the synthesis step */
                    for (k = 0; k <= fftFrameSize2; k++)
                    {

                        /* get magnitude and true frequency from synthesis arrays */
                        magn = gSynMagn[k];
                        tmp = gSynFreq[k];

                        /* subtract bin mid frequency */
                        tmp -= k * freqPerBin;

                        /* get bin deviation from freq deviation */
                        tmp /= freqPerBin;

                        /* take osamp into account */
                        tmp = 2.0 * Math.PI * tmp / osamp;

                        /* add the overlap phase advance back in */
                        tmp += k * expct;

                        /* accumulate delta phase to get bin phase */
                        gSumPhase[k] += (float)tmp;
                        phase = gSumPhase[k];

                        /* get real and imag part and re-interleave */
                        gFFTworksp[2 * k] = (float)(magn * Math.Cos(phase));
                        gFFTworksp[2 * k + 1] = (float)(magn * Math.Sin(phase));
                    }

                    /* zero negative frequencies */
                    for (k = fftFrameSize + 2; k < 2 * fftFrameSize; k++) gFFTworksp[k] = 0.0f;

                    /* do inverse transform */
                    smbFft(gFFTworksp, fftFrameSize, 1);

                    /* do windowing and add to output accumulator */
                    for (k = 0; k < fftFrameSize; k++)
                    {
                        window = -0.5 * Math.Cos(2.0 * Math.PI * k / fftFrameSize) + 0.5;
                        gOutputAccum[k] += (float)(2.0 * window * gFFTworksp[2 * k] / (fftFrameSize2 * osamp));
                    }
                    for (k = 0; k < stepSize; k++) gOutFIFO[k] = gOutputAccum[k];

                    /* shift accumulator */
                    for (k = 0; k < fftFrameSize; k++)
                        gOutputAccum[k] = gOutputAccum[k + stepSize];

                    /* move input FIFO */
                    for (k = 0; k < inFifoLatency; k++) gInFIFO[k] = gInFIFO[k + stepSize];
                }
            }
        }

        /* 
	        FFT routine, (C)1996 S.M.Bernsee. Sign = -1 is FFT, 1 is iFFT (inverse)
	        Fills fftBuffer[0...2*fftFrameSize-1] with the Fourier transform of the
	        time domain data in fftBuffer[0...2*fftFrameSize-1]. The FFT array takes
	        and returns the cosine and sine parts in an interleaved manner, ie.
	        fftBuffer[0] = cosPart[0], fftBuffer[1] = sinPart[0], asf. fftFrameSize
	        must be a power of 2. It expects a complex input signal (see footnote 2),
	        ie. when working with 'common' audio signals our input signal has to be
	        passed as {in[0],0.,in[1],0.,in[2],0.,...} asf. In that case, the transform
	        of the frequencies of interest is in fftBuffer[0...fftFrameSize].
        */
        private void smbFft(float[] fftBuffer, long fftFrameSize, long sign)
        {
            long i, bitm, j, le, le2, k;
            for (i = 2; i < 2 * fftFrameSize - 2; i += 2)
            {
                for (bitm = 2, j = 0; bitm < 2 * fftFrameSize; bitm <<= 1)
                {
                    if ((i & bitm) != 0) j++;
                    j <<= 1;
                }
                if(i < j)
                {
                    float p1 = fftBuffer[i], p2 = fftBuffer[j];
                    fftBuffer[i] = p2;
                    fftBuffer[j] = p1;
                    p1 = fftBuffer[i + 1];
                    p2 = fftBuffer[j + 1];
                    fftBuffer[i + 1] = p2;
                    fftBuffer[j + 1] = p1;
                }
            }
            for(k = 0, le = 2; k < (long)(Math.Log(fftFrameSize) / Math.Log(2.0) + 0.5); k++)
            {
                le <<= 1;
                le2 = le >> 1;
                float ur = 1.0f, ui = 0.0f;
                float arg = (float)Math.PI / (le2 >> 1);
                float wr = (float)Math.Cos(arg);
                float wi = (float)(sign * Math.Sin(arg));
                float tr, ti;
                for (j = 0; j < le2; j += 2)
                {
                    long p1r = j, p1i = p1r + 1;
                    long p2r = p1r + le2, p2i = p2r + 1;
                    for (i = j; i < 2 * fftFrameSize; i += le)
                    {
                        tr = fftBuffer[p2r] * ur - fftBuffer[p2i] * ui;
                        ti = fftBuffer[p2r] * ui + fftBuffer[p2i] * ur;
                        fftBuffer[p2r] = fftBuffer[p1r] - tr;
                        fftBuffer[p2i] = fftBuffer[p1i] - ti;
                        fftBuffer[p1r] += tr;
                        fftBuffer[p1i] += ti;
                        p1r += le; p1i += le;
                        p2r += le; p2i += le;
                    }
                    tr = ur * wr - ui * wi;
                    ui = ur * wi + ui * wr;
                    ur = tr;
                }
            }
        }
    }
}
