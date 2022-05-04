package com.example.microphone.audio

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class AudioBuffer
{
    // set buffer size for latency
    private val BUFFER_SIZE = 5
    // actual buffer of byte arrays (FIFO with queue)
    private val buffer = ArrayDeque<ByteArray>()
    // mutex for coroutines
    private val mutex = Mutex()
    // insert new data, remove overflowed data
    suspend fun push(data : ByteArray)
    {
        mutex.withLock {
            buffer.addLast(data)
            while(buffer.size > BUFFER_SIZE)
                buffer.removeFirst()
        }
    }
    // retrieve data, may be empty
    suspend fun poll() : ByteArray?
    {
        mutex.withLock {
            return buffer.removeFirstOrNull()
        }
    }
    // reset buffer
    suspend fun reset()
    {
        mutex.withLock {
            buffer.clear()
        }
    }
}