package com.example.androidMic.domain.streaming

import com.example.androidMic.domain.service.AudioPacket
import kotlinx.coroutines.flow.Flow

interface Streamer {
    fun connect(): Boolean
    fun disconnect(): Boolean
    fun shutdown()
    fun start(audioStream: Flow<AudioPacket>)
    fun getInfo(): String
    fun isAlive(): Boolean

    companion object {
        const val DEVICE_CHECK = "AndroidMicCheck"
        const val DEVICE_CHECK_EXPECT = "AndroidMicCheckAck"
        const val BUFFER_SIZE = 1024
    }
}