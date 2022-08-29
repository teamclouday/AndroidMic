using System;
using System.Threading;

namespace AndroidMic.Audio
{
    // implementation of circular buffer
    public class AudioBuffer
    {
        public const int Capacity = 5 * 1024;
        public readonly byte[] Buffer = new byte[Capacity];

        private int regionSize = 0;
        private int regionLeft = 0;
        private int regionRight = 0;

        private static readonly object mutex = new object();

        /// <summary>
        /// Open buffer region to read from
        /// </summary>
        /// <param name="requestSize"></param>
        /// <param name="actualSize"></param>
        /// <param name="offset"></param>
        public void OpenReadRegion(int requestSize, out int actualSize, out int offset)
        {
            Monitor.Enter(mutex);
            actualSize = Math.Min(Math.Min(requestSize, regionSize), Capacity - regionLeft);
            offset = regionLeft;
        }

        /// <summary>
        /// Close buffer read region
        /// </summary>
        /// <param name="readSize"></param>
        public void CloseReadRegion(int readSize)
        {
            regionLeft = (regionLeft + readSize) % Capacity;
            regionSize -= Math.Min(readSize, regionSize);
            Monitor.Exit(mutex);
        }

        /// <summary>
        /// Open buffer region to write to
        /// </summary>
        /// <param name="requestSize"></param>
        /// <param name="actualSize"></param>
        /// <param name="offset"></param>
        public void OpenWriteRegion(int requestSize, out int actualSize, out int offset)
        {
            Monitor.Enter(mutex);
            actualSize = Math.Min(Math.Min(requestSize, Capacity - regionSize), Capacity - regionRight);
            offset = regionRight;
        }

        /// <summary>
        /// Close buffer write region
        /// </summary>
        /// <param name="writeSize"></param>
        public void CloseWriteRegion(int writeSize)
        {
            regionRight = (regionRight + writeSize) % Capacity;
            regionSize = Math.Min(regionSize + writeSize, Capacity);
            Monitor.Exit(mutex);
        }

        /// <summary>
        /// Get buffer size
        /// </summary>
        /// <returns></returns>
        public int Size()
        {
            lock (mutex)
            {
                return regionSize;
            }
        }

        /// <summary>
        /// Whether buffer is empty
        /// </summary>
        /// <returns></returns>
        public bool IsEmpty()
        {
            lock (mutex)
            {
                return regionSize == 0;
            }
        }

        /// <summary>
        /// Clear buffer
        /// </summary>
        public void Clear()
        {
            lock (mutex)
            {
                regionLeft = 0;
                regionRight = 0;
                regionSize = 0;
            }
        }
    }
}
