using System;
using System.Windows;
using System.Windows.Media;
using System.Windows.Shapes;
using System.Runtime.InteropServices;
using NAudio.Wave;
using libspeexdsp;

namespace AndroidMic.Audio
{
    // use libspeexdsp preprocessor
    // reference: iaxclient/lib/audio_encode.c
    public class FilterSpeexDSP : IWaveProvider, IDisposable
    {
        public enum ConfigTypes
        {
            ConfigDenoise = 0,
            ConfigAGC = 1,
            ConfigEcho = 2,
            ConfigVAD = 3
        }

        private readonly int FRAME_SIZE = 1600; // 0.1 * 16000
        private readonly int FILTER_LEN = 12800; // or 100

        // configs specifically chosen for this app
        private readonly int VAD_PROB_START = 85;
        private readonly int VAD_PROB_CONTINUE = 70;

        private readonly short[] audioBuffer;
        private readonly short[] echoPlayBuffer;
        private readonly short[] echoOutBuffer;

        private readonly IWaveProvider source;
        private Ellipse indicator;
        private readonly Brush indicatorOn;
        private readonly Brush indicatorOff;

        private readonly IntPtr SpeexPreprocessState;
        private readonly IntPtr SpeexEchoState;
        private readonly IntPtr StateUpdate;

        public FilterSpeexDSP(IWaveProvider source)
        {
            this.source = source;
            audioBuffer = new short[FRAME_SIZE];
            echoPlayBuffer = new short[FRAME_SIZE];
            echoOutBuffer = new short[FRAME_SIZE];
            SpeexPreprocessState = SpeexPreprocess.speex_preprocess_state_init(FRAME_SIZE, WaveFormat.SampleRate);
            SpeexEchoState = SpeexEcho.speex_echo_state_init(FRAME_SIZE, FILTER_LEN);
            StateUpdate = Marshal.AllocHGlobal(sizeof(int));
            _EnabledDenoise = false;
            _EnabledAGC = false;
            _EnabledVAD = false;
            _EnabledEcho = false;
            var converter = new BrushConverter();
            indicatorOn = converter.ConvertFromString("#FFFF9494") as SolidColorBrush;
            indicatorOff = converter.ConvertFromString("#94fffa") as SolidColorBrush;
        }

        // read call
        public int Read(byte[] buffer, int offset, int sampleCount)
        {
            int samplesRead = source.Read(buffer, offset, sampleCount);
            // skip if no filters are enabled
            if (!(EnabledDenoise || EnabledAGC || EnabledEcho || EnabledVAD)) return samplesRead;
            int toRead = samplesRead;
            while(toRead > 0)
            {
                // copy buffer
                int nextRead = Math.Min(toRead, FRAME_SIZE);
                Buffer.BlockCopy(buffer, offset, audioBuffer, 0, nextRead);
                toRead -= nextRead;
                // process audio
                if (EnabledEcho)
                {
                    SpeexEcho.speex_echo_cancellation(SpeexEchoState, audioBuffer, echoPlayBuffer, echoOutBuffer);
                    Buffer.BlockCopy(echoOutBuffer, 0, audioBuffer, 0, nextRead);
                }
                InSpeech = SpeexPreprocess.speex_preprocess_run(SpeexPreprocessState, audioBuffer) == 1;
                // copy back
                Buffer.BlockCopy(audioBuffer, 0, buffer, offset, nextRead);
            }
            InSpeech = InSpeech && EnabledVAD;
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                UpdateIndicator();
            }));
            return samplesRead;
        }

        // dispose object
        public void Dispose()
        {
            SpeexEcho.speex_echo_state_destroy(SpeexEchoState);
            SpeexPreprocess.speex_preprocess_state_destroy(SpeexPreprocessState);
            Marshal.FreeHGlobal(StateUpdate);
        }

        // set indicator
        public void SetIndicator(Ellipse e)
        {
            indicator = e;
        }

        // update indicator
        private void UpdateIndicator()
        {
            if (indicator == null) return;
            indicator.Fill = InSpeech ? indicatorOn : indicatorOff;
        }

        public WaveFormat WaveFormat => source.WaveFormat;

        private bool _EnabledDenoise;
        public bool EnabledDenoise
        {
            get
            {
                return _EnabledDenoise;
            }
            set
            {
                _EnabledDenoise = value;
                if(SpeexPreprocessState != null)
                {
                    Marshal.WriteInt32(StateUpdate, _EnabledDenoise ? 1 : 2);
                    // Turns denoising on(1) or off(2)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_DENOISE,
                        StateUpdate);
                }
            }
        }

        private bool _EnabledAGC;
        public bool EnabledAGC
        {
            get
            {
                return _EnabledAGC;
            }
            set
            {
                _EnabledAGC = value;
                if(SpeexPreprocessState != null)
                {
                    Marshal.WriteInt32(StateUpdate, _EnabledAGC ? 1 : 2);
                    // Turns automatic gain control (AGC) on(1) or off(2)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_AGC,
                        StateUpdate);
                }
            }
        }

        private bool _EnabledVAD;
        public bool EnabledVAD
        {
            get
            {
                return _EnabledVAD;
            }
            set
            {
                _EnabledVAD = value;
                if (SpeexPreprocessState != null)
                {
                    Marshal.WriteInt32(StateUpdate, _EnabledVAD ? 1 : 2);
                    // Turns voice activity detector (VAD) on(1) or off(2)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_VAD,
                        StateUpdate);
                    if(_EnabledVAD)
                    {
                        // config VAD probs
                        Marshal.WriteInt32(StateUpdate, VAD_PROB_START);
                        SpeexPreprocess.speex_preprocess_ctl(
                            SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_PROB_START,
                            StateUpdate);
                        Marshal.WriteInt32(StateUpdate, VAD_PROB_CONTINUE);
                        SpeexPreprocess.speex_preprocess_ctl(
                            SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_PROB_CONTINUE,
                            StateUpdate);
                    }
                }
            }
        }

        private bool _EnabledEcho;
        public bool EnabledEcho
        {
            get
            {
                return _EnabledEcho;
            }
            set
            {
                _EnabledEcho = value;
                if (SpeexPreprocessState != null)
                {
                    // Set the associated echo canceller for residual echo suppression
                    // (pointer or NULL for no residual echo suppression)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_STATE,
                        _EnabledEcho ? SpeexEchoState : IntPtr.Zero);
                }
            }
        }

        public bool InSpeech { get; private set; }
    }
}
