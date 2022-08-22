using AndroidMic.Audio;

namespace AndroidMic.Streaming
{
    public abstract class Streamer
    {
        // shutdown server
        public abstract void Shutdown();
        // process data
        public abstract void Process(AudioBuffer sharedBuffer);
        // check if streamer is alive
        public abstract bool IsAlive();
        // get client info
        public abstract string GetClientInfo();
        // get server info
        public abstract string GetServerInfo();

        public enum ServerStatus
        {
            DEFAULT,
            LISTENING,
            CONNECTED
        }
        public ServerStatus Status { get; protected set; } = ServerStatus.DEFAULT;

        public static readonly string DEVICE_CHECK_EXPECTED = "AndroidMicCheck";
        public static readonly string DEVICE_CHECK = "AndroidMicCheckAck";
        public static readonly int MAX_WAIT_TIME = 1500;
        public static readonly int MIN_WAIT_TIME = 50;
        public static readonly int BUFFER_SIZE = 1024;
    }
}
