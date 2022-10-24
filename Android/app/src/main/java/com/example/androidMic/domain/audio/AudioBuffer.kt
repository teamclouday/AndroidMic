package com.example.androidMic.domain.audio

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlin.math.min

// thread safe circular buffer
class AudioBuffer {
    private var regionLeft = 0
    private var regionRight = 0
    private var regionSize = 0

    val capacity = 5 * 1024
    val buffer = ByteArray(capacity)

    private val mutex = Mutex()

    // get memory region to read from
    suspend fun openReadRegion(requestSize: Int): Pair<Int, Int> {
        mutex.lock()
        val actualSize = min(min(requestSize, regionSize), capacity - regionLeft)
        val offset = regionLeft
        return Pair(actualSize, offset)
    }

    // mark this region as read
    fun closeReadRegion(actualSize: Int) {
        regionLeft = (regionLeft + actualSize) % capacity
        regionSize -= min(actualSize, regionSize)
        mutex.unlock()
    }

    // get memory region to write to
    suspend fun openWriteRegion(requestSize: Int): Pair<Int, Int> {
        mutex.lock()
        val actualSize = min(min(requestSize, capacity - regionSize), capacity - regionRight)
        val offset = regionRight
        return Pair(actualSize, offset)
    }

    // mark this region as written
    fun closeWriteRegion(actualSize: Int) {
        regionRight = (regionRight + actualSize) % capacity
        regionSize = min(regionSize + actualSize, capacity)
        mutex.unlock()
    }

    // get valid buffer size
    suspend fun size(): Int {
        mutex.withLock {
            return regionSize
        }
    }

    // check if buffer is empty
    suspend fun isEmpty(): Boolean {
        mutex.withLock {
            return regionSize == 0
        }
    }

    suspend fun clear() {
        mutex.withLock {
            regionLeft = 0
            regionRight = 0
            regionSize = 0
        }
    }
}