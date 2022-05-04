using System.Collections.Generic;

namespace AndroidMic.Audio
{
    public class AudioBuffer
    {
        public const int MAX_BUFFER_SIZE = 5;
        private readonly Queue<byte[]> buffer = new Queue<byte[]>();
        private readonly object toLock = new object();

        public void push(byte[] data)
        {
            lock(toLock)
            {
                buffer.Enqueue(data);
                while (buffer.Count > MAX_BUFFER_SIZE) buffer.Dequeue();
            }
        }

        public byte[] poll()
        {
            lock(this)
            {
                if (buffer.Count > 0) return buffer.Dequeue();
                else return null;
            }
        }
    }
}
