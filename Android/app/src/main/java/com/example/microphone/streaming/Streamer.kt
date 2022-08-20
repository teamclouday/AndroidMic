package com.example.microphone.streaming

import com.example.microphone.audio.AudioBuffer
import java.net.InetSocketAddress

interface Streamer {
    fun connect(): Boolean
    fun disconnect(): Boolean
    fun shutdown()
    suspend fun stream(audioBuffer: AudioBuffer)
    fun getInfo(): String
    fun updateAddress(address: InetSocketAddress)
    fun isAlive(): Boolean

    companion object {
        val DEVICE_CHECK = "AndroidMicCheck"
        val DEVICE_CHECK_EXPECT = "AndroidMicCheckAck"
    }
}