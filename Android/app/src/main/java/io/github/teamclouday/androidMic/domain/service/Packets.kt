package io.github.teamclouday.androidMic.domain.service

// definition of an audio packet
data class AudioPacket(
    val buffer: ByteArray,
    val sampleRate: Int,
    val channelCount: Int,
    val audioFormat: Int,
)