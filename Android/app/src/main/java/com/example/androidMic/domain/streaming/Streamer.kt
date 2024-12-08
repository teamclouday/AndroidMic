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
}